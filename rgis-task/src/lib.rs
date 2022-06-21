#![warn(
    clippy::unwrap_used,
    clippy::cast_lossless,
    clippy::unimplemented,
    clippy::expect_used
)]

use bevy::prelude::Component;
use std::{any, future, pin};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(check_system)
            .insert_resource(FinishedTasks { outcomes: vec![] });
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type PerformReturn<Output> =
    pin::Pin<Box<dyn future::Future<Output = Output> + Send + 'static>>;
#[cfg(target_arch = "wasm32")]
pub type PerformReturn<Output> = pin::Pin<Box<dyn future::Future<Output = Output> + 'static>>;

pub trait Task: any::Any + Sized + Send + Sync + 'static {
    type Outcome: any::Any + Send + Sync;

    fn name(&self) -> String;

    fn perform(self) -> PerformReturn<Self::Outcome>;

    fn spawn(
        self,
        pool: &bevy::tasks::AsyncComputeTaskPool,
        commands: &mut bevy::ecs::system::Commands,
    ) {
        let (sender, receiver) = async_channel::unbounded::<OutcomePayload>();

        let task_name = self.name();
        let in_progress_task = InProgressTask {
            task_name: task_name.clone(),
        };

        pool.spawn(async move {
            bevy::log::info!("Starting task '{}'", task_name);
            let outcome = self.perform().await;
            bevy::log::info!("Completed task '{}'", task_name);
            if let Err(e) = sender
                .send((any::TypeId::of::<Self>(), Box::new(outcome)))
                .await
            {
                bevy::log::error!(
                    "Failed to send result from task {} back to main thread: {:?}",
                    task_name,
                    e
                );
            }
        })
        .detach();

        commands
            .spawn()
            .insert(in_progress_task)
            .insert(InProgressTaskOutcomeReceiver(receiver));
    }
}

fn check_system(
    query: bevy::ecs::system::Query<(&InProgressTaskOutcomeReceiver, bevy::ecs::entity::Entity)>,
    mut commands: bevy::ecs::system::Commands,
    mut finished_tasks: bevy::ecs::system::ResMut<FinishedTasks>,
) {
    query.for_each(|(receiver, entity)| {
        if let Ok(outcome) = receiver.0.try_recv() {
            bevy::log::info!("Task finished");
            commands.entity(entity).despawn();
            finished_tasks.outcomes.push(outcome);
        }
    })
}

// (<task type ID>, <task outcome value>)
type OutcomePayload = (any::TypeId, Box<dyn any::Any + Send + Sync>);

#[derive(Component)]
pub struct InProgressTask {
    pub task_name: String,
}

#[derive(Component)]
pub struct InProgressTaskOutcomeReceiver(async_channel::Receiver<OutcomePayload>);

pub struct FinishedTasks {
    outcomes: Vec<OutcomePayload>,
}

impl FinishedTasks {
    pub fn take_next<T: Task>(&mut self) -> Option<T::Outcome> {
        let next = self
            .outcomes
            .iter_mut()
            .enumerate()
            .filter(|(_i, (type_id, outcome))| {
                outcome.is::<T::Outcome>() && any::TypeId::of::<T>() == *type_id
            })
            .map(|(i, _outcome)| i)
            .next();
        match next {
            Some(index) => {
                let (_type_id, x) = self.outcomes.remove(index);
                Some(*x.downcast::<T::Outcome>().unwrap())
            }
            None => None,
        }
    }
}
