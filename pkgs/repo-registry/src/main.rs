// repo-registry: Which repositories are tracked (commit platform, pkgs/repo-registry)
//! Registry of tracked repositories (SeaORM over SQLite): which repos the
//! platform collects activity for, and their display names.

use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue::Set, Database, DatabaseConnection, QueryOrder, Schema};

mod repository {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "repositories")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub url: String,
        pub title: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(url).await?;
    let backend = db.get_database_backend();
    db.execute(&Schema::new(backend).create_table_from_entity(repository::Entity))
        .await?;
    Ok(db)
}

async fn add(db: &DatabaseConnection, url: &str, title: &str) -> Result<repository::Model, DbErr> {
    repository::ActiveModel {
        url: Set(url.into()),
        title: Set(title.into()),
        ..Default::default()
    }
    .insert(db)
    .await
}

async fn all(db: &DatabaseConnection) -> Result<Vec<repository::Model>, DbErr> {
    repository::Entity::find()
        .order_by_asc(repository::Column::Title)
        .all(db)
        .await
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db = connect("sqlite::memory:").await?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    for pair in args.chunks(2) {
        if let [url, title] = pair {
            add(&db, url, title).await?;
        }
    }
    for r in all(&db).await? {
        println!("{}\t{}", r.title, r.url);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stores_and_sorts_by_title() {
        let db = connect("sqlite::memory:").await.unwrap();
        add(&db, "https://github.com/o/beta", "Beta").await.unwrap();
        add(&db, "https://github.com/o/alpha", "Alpha")
            .await
            .unwrap();
        let titles: Vec<_> = all(&db)
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.title)
            .collect();
        assert_eq!(titles, ["Alpha", "Beta"]);
    }
}
