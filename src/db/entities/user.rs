use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    #[sea_orm(unique, indexed)]
    pub wallet_address: String,

    #[sea_orm(unique, nullable, indexed)]
    pub username: Option<String>,

    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::canvas::Entity")]
    Canvases,

    #[sea_orm(has_many = "super::canvas_collaborator::Entity")]
    Collaborations,

    #[sea_orm(has_many = "super::pixel::Entity")]
    Pixels,
}

impl Related<super::canvas::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Canvases.def()
    }
}

impl Related<super::canvas_collaborator::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Collaborations.def()
    }
}

impl Related<super::pixel::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Pixels.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
