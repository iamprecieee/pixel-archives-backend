use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "canvases")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    #[sea_orm(indexed)]
    pub owner_id: Uuid,

    pub name: String,

    #[sea_orm(unique)]
    pub invite_code: String,

    pub state: CanvasState,

    #[sea_orm(nullable)]
    pub canvas_pda: Option<String>,

    #[sea_orm(nullable)]
    pub mint_address: Option<String>,

    pub total_escrowed: i64,

    pub created_at: DateTimeUtc,

    #[sea_orm(nullable)]
    pub published_at: Option<DateTimeUtc>,

    #[sea_orm(nullable)]
    pub minted_at: Option<DateTimeUtc>,
}

#[derive(Clone, Debug, Default, EnumIter, DeriveActiveEnum, PartialEq)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum CanvasState {
    #[default]
    #[sea_orm(string_value = "draft")]
    Draft,

    #[sea_orm(string_value = "publishing")]
    Publishing,

    #[sea_orm(string_value = "published")]
    Published,

    #[sea_orm(string_value = "mint_pending")]
    MintPending,

    #[sea_orm(string_value = "minting")]
    Minting,

    #[sea_orm(string_value = "minted")]
    Minted,
}

impl CanvasState {
    pub fn is_valid_transition(&self, target: &CanvasState) -> bool {
        use CanvasState::*;

        matches!(
            (self, target),
            // Forward transitions
            (Draft, Publishing)
                | (Publishing, Published)
                | (Published, MintPending)
                | (MintPending, Minting)
                | (Minting, Minted)
                // Rollback/failure transitions
                | (Publishing, Draft)      // Publish failed/cancelled
                | (MintPending, Published) // Mint cancelled
                | (Minting, Published) // Mint failed
        )
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::OwnerId",
        to = "super::user::Column::Id"
    )]
    Owner,

    #[sea_orm(has_many = "super::canvas_collaborator::Entity")]
    CanvasCollaborator,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Owner.def()
    }
}

impl Related<super::canvas_collaborator::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CanvasCollaborator.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
