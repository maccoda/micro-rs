use diesel;
use diesel::prelude::*;
use super::schema::tasks;

#[derive(Deserialize, Serialize, Queryable, Clone)]
pub struct Task {
    id: i32,
    task: String,
    completed: bool,
}

impl Task {
    pub fn all(conn: &PgConnection) -> Vec<Task> {
        use schema::tasks::dsl::*;
        tasks.load::<Task>(conn).expect("Could not load tasks")
    }

    pub fn create(conn: &PgConnection, task: NewTask) {
        use schema::tasks;
        diesel::insert_into(tasks::table)
            .values(&task)
            .execute(conn)
            .expect("Unable to insert");
    }

    pub fn update(conn: &PgConnection, task_update: Task) -> i32 {
        use schema::tasks::dsl::*;
        diesel::update(tasks.find(task_update.id))
            .set((
                task.eq(task_update.task),
                completed.eq(task_update.completed),
            ))
            .execute(conn)
            .expect("Failed to update");
        task_update.id
    }

    pub fn delete(conn: &PgConnection, task_id: i32) {
        use schema::tasks::dsl::*;
        diesel::delete(tasks.find(task_id))
            .execute(conn)
            .expect("Failed to delete");
    }
}

#[derive(Deserialize, Serialize, Insertable)]
#[table_name = "tasks"]
pub struct NewTask {
    task: String,
    completed: bool,
}
