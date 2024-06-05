use utoipa::OpenApi;

use crate::handlers;
use crate::models::api::request::{
    AddBlock, AlterBlock, AlterBoard, ChangeBlock, ChangeState, MoveBlock,
};
use crate::models::api::response::{Board, Solution, Solved};
use crate::models::game::blocks::{Block, Positioned};
use crate::models::game::board::State;
use crate::models::game::moves::{FlatBoardMove, FlatMove};
use crate::models::game::utils::Position;

#[derive(OpenApi)]
#[openapi(
    info(title = "Klotski API", version = "0.1.0",),
    paths(
        handlers::block::add,
        handlers::block::alter,
        handlers::block::remove,
        handlers::board::new,
        handlers::board::alter,
        handlers::board::delete,
        handlers::board::solve,
    ),
    components(schemas(
        AddBlock,
        AlterBlock,
        AlterBoard,
        Block,
        Board,
        ChangeBlock,
        ChangeState,
        FlatBoardMove,
        FlatMove,
        MoveBlock,
        Positioned,
        Position,
        Solution,
        Solved,
        State
    ),)
)]
pub struct ApiDoc;
