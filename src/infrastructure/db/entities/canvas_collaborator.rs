use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "canvas_collaborators")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub canvas_id: Uuid,

    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,

    pub joined_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::canvas::Entity",
        from = "Column::CanvasId",
        to = "super::canvas::Column::Id"
    )]
    Canvas,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::canvas::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Canvas.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
