//! Action system for long-running tasks
//! 
//! Provides a way to handle complex, long-running operations with feedback and cancellation.

use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::Debug;
use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock, broadcast};
use anyhow::{Result, anyhow};
use uuid::Uuid;

/// Trait for action goals
pub trait ActionGoal: Clone + Send + Sync + Debug + 'static {
    type Feedback: Clone + Send + Sync + Debug + 'static;
    type Result: Clone + Send + Sync + Debug + 'static;
    
    fn goal_id(&self) -> Uuid;
    fn action_name(&self) -> String;
}

/// Trait for action handlers
#[async_trait]
pub trait ActionHandler<Goal: ActionGoal>: Send + Sync {
    async fn execute(
        &self,
        goal: Goal,
        feedback_tx: mpsc::Sender<Goal::Feedback>,
        cancel_rx: &mut broadcast::Receiver<()>,
    ) -> Goal::Result;
}

/// Status of an action goal
#[derive(Debug, Clone, PartialEq)]
pub enum ActionGoalStatus {
    Pending,
    Active,
    Succeeded,
    Aborted,
    Canceled,
}

/// Handle for an action goal
#[derive(Debug)]
pub struct ActionGoalHandle<Goal: ActionGoal> {
    pub goal_id: Uuid,
    pub goal: Goal,
    pub status: ActionGoalStatus,
    pub result_rx: Option<tokio::sync::oneshot::Receiver<Goal::Result>>,
    pub feedback_rx: mpsc::Receiver<Goal::Feedback>,
    pub cancel_tx: broadcast::Sender<()>,
}

impl<Goal: ActionGoal> ActionGoalHandle<Goal> {
    /// Wait for the result
    pub async fn get_result(&mut self) -> Result<Goal::Result> {
        if let Some(result_rx) = self.result_rx.take() {
            result_rx.await.map_err(|e| anyhow!("Failed to receive result: {}", e))
        } else {
            Err(anyhow!("Result already consumed"))
        }
    }
    
    /// Try to receive feedback (non-blocking)
    pub fn try_recv_feedback(&mut self) -> Result<Option<Goal::Feedback>> {
        match self.feedback_rx.try_recv() {
            Ok(feedback) => Ok(Some(feedback)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                Err(anyhow!("Feedback channel disconnected"))
            }
        }
    }
    
    /// Cancel the action
    pub async fn cancel(&self) -> Result<()> {
        self.cancel_tx.send(())?;
        Ok(())
    }
}

/// Simple action goal for testing
#[derive(Debug, Clone)]
pub struct SimpleActionGoal {
    pub id: Uuid,
    pub action_name: String,
    pub duration_secs: u64,
}

impl ActionGoal for SimpleActionGoal {
    type Feedback = SimpleActionFeedback;
    type Result = SimpleActionResult;
    
    fn goal_id(&self) -> Uuid {
        self.id
    }
    
    fn action_name(&self) -> String {
        self.action_name.clone()
    }
}

#[derive(Debug, Clone)]
pub struct SimpleActionFeedback {
    pub progress: f32,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct SimpleActionResult {
    pub success: bool,
    pub message: String,
}

/// Action client for sending goals to actions
#[derive(Debug)]
pub struct ActionClient<Goal: ActionGoal> {
    action_name: String,
    goal_tx: mpsc::Sender<ActionCommand<Goal>>,
}

#[derive(Debug)]
pub enum ActionCommand<Goal: ActionGoal> {
    ExecuteGoal {
        goal: Goal,
        feedback_tx: mpsc::Sender<Goal::Feedback>,
        result_tx: tokio::sync::oneshot::Sender<Goal::Result>,
        cancel_rx: broadcast::Receiver<()>,
    },
}

impl<Goal: ActionGoal> ActionClient<Goal> {
    pub fn new(action_name: String, goal_tx: mpsc::Sender<ActionCommand<Goal>>) -> Self {
        Self {
            action_name,
            goal_tx,
        }
    }
    
    /// Send a goal to the action server
    pub async fn send_goal(&self, goal: Goal) -> Result<ActionGoalHandle<Goal>> {
        let (feedback_tx, feedback_rx) = mpsc::channel(100);
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();
        let (cancel_tx, cancel_rx) = broadcast::channel(1);
        
        let goal_id = goal.goal_id();
        let goal_clone = goal.clone();
        
        let command = ActionCommand::ExecuteGoal {
            goal,
            feedback_tx,
            result_tx,
            cancel_rx,
        };
        
        self.goal_tx.send(command).await
            .map_err(|_| anyhow!("Action server {} is not available", self.action_name))?;
        
        Ok(ActionGoalHandle {
            goal_id,
            goal: goal_clone,
            status: ActionGoalStatus::Pending,
            result_rx: Some(result_rx),
            feedback_rx,
            cancel_tx,
        })
    }
}

/// Action server for handling goals
#[derive(Debug)]
pub struct ActionServer {
    actions: Arc<RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
}

impl ActionServer {
    pub fn new() -> Self {
        Self {
            actions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register_action<Goal, H>(&self, action_name: String, handler: H) -> Result<ActionClient<Goal>>
    where
        Goal: ActionGoal + 'static,
        H: ActionHandler<Goal> + 'static,
    {
        let (goal_tx, mut goal_rx) = mpsc::channel::<ActionCommand<Goal>>(100);
        let client = ActionClient::new(action_name.clone(), goal_tx.clone());
        
        // Store the sender for this action
        self.actions.write().await.insert(action_name.clone(), Box::new(goal_tx));
        
        let handler = Arc::new(handler);
        
        // Spawn a task to handle goals
        tokio::spawn(async move {
            while let Some(command) = goal_rx.recv().await {
                match command {
                    ActionCommand::ExecuteGoal { goal, feedback_tx, result_tx, mut cancel_rx } => {
                        let result = handler.execute(goal, feedback_tx, &mut cancel_rx).await;
                        let _ = result_tx.send(result);
                    }
                }
            }
        });
        
        log::info!("Registered action: {}", action_name);
        Ok(client)
    }
}

// Example action handler
pub struct SimpleActionHandler;

#[async_trait]
impl ActionHandler<SimpleActionGoal> for SimpleActionHandler {
    async fn execute(
        &self,
        goal: SimpleActionGoal,
        feedback_tx: mpsc::Sender<SimpleActionFeedback>,
        cancel_rx: &mut broadcast::Receiver<()>,
    ) -> SimpleActionResult {
        log::info!("Starting action: {}", goal.action_name);
        
        let total_steps = goal.duration_secs;
        for step in 0..total_steps {
            // Check for cancellation
            if cancel_rx.try_recv().is_ok() {
                log::info!("Action {} was canceled", goal.action_name);
                return SimpleActionResult {
                    success: false,
                    message: "Action was canceled".to_string(),
                };
            }
            
            // Send feedback
            let progress = step as f32 / total_steps as f32;
            let feedback = SimpleActionFeedback {
                progress,
                message: format!("Step {} of {}", step + 1, total_steps),
            };
            
            if feedback_tx.send(feedback).await.is_err() {
                log::warn!("Failed to send feedback for action {}", goal.action_name);
            }
            
            // Simulate work
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        log::info!("Action {} completed successfully", goal.action_name);
        SimpleActionResult {
            success: true,
            message: "Action completed successfully".to_string(),
        }
    }
}