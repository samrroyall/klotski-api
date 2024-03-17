use utoipa::OpenApi;

use crate::handlers::board as board_handlers;
use crate::models::api::request::{
    AddBlock, AlterBlock, AlterBoard, ChangeBlock, ChangeState, MoveBlock, NewBoard,
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
        board_handlers::add_block,
        board_handlers::alter_block,
        board_handlers::remove_block,
        board_handlers::new,
        board_handlers::alter,
        board_handlers::delete,
        board_handlers::solve,
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
        NewBoard,
        Positioned,
        Position,
        Solution,
        Solved,
        State
    ),)
)]
pub struct ApiDoc;
