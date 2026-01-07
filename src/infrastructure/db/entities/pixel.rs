use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "pixels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub canvas_id: Uuid,

    #[sea_orm(primary_key, auto_increment = false)]
    pub x: i16,

    #[sea_orm(primary_key, auto_increment = false)]
    pub y: i16,

    pub color: i16,

    #[sea_orm(nullable, indexed)]
    pub owner_id: Option<Uuid>,

    pub price_lamports: i64,

    pub updated_at: DateTimeUtc,
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
        from = "Column::OwnerId",
        to = "super::user::Column::Id"
    )]
    Owner,
}

impl Related<super::canvas::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Canvas.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Owner.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
