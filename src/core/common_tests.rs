use super::common::*;
use serde_json;

#[test]
fn player_serialize() {
    let mut player = Player::new(1);

    let serialized = serde_json::to_string(&player).unwrap();
    assert_eq!(serialized, r#"{"id":1}"#);

    player.defeated_at = Some(3);
    let serialized = serde_json::to_string(&player).unwrap();
    assert_eq!(serialized, r#"{"id":1,"defeated_at":3}"#);

    player.owned_tiles = 5;
    let serialized = serde_json::to_string(&player).unwrap();
    assert_eq!(serialized, r#"{"id":1,"owned_tiles":5,"defeated_at":3}"#);
}

#[test]
fn action_serialize() {
    let action = Action::Move(Move {
        player: 1,
        from: 42,
        direction: Direction::Left,
    });
    let serialized = serde_json::to_string(&action).unwrap();
    assert_eq!(
        serialized,
        r#"{"type":"move","from":42,"direction":"left"}"#
    );

    let action = Action::Resign;
    let serialized = serde_json::to_string(&action).unwrap();
    assert_eq!(serialized, r#"{"type":"resign"}"#);

    let action = Action::CancelMoves;
    let serialized = serde_json::to_string(&action).unwrap();
    assert_eq!(serialized, r#"{"type":"cancel_moves"}"#);
}

#[test]
fn player() {
    let mut player = Player::new(1);
    // by default the player does not own any tile, so cannot move
    assert!(!player.can_move());

    // if we give him one, it should be able to move
    player.owned_tiles = 1;
    assert!(player.can_move());

    // if the playe is defeated it should not be able to move anymore
    player.defeated_at = Some(99);
    assert!(!player.can_move());
}

#[test]
fn tile() {
    fn check_dirty_and_clean(tile: &mut Tile) {
        assert!(tile.is_dirty());
        tile.set_clean();
        assert!(!tile.is_dirty());
    }

    let mut tile = Tile::new();
    assert!(tile.owner().is_none());
    assert!(tile.is_mountain());
    assert!(!tile.is_open());
    assert!(!tile.is_general());
    assert!(!tile.is_city());
    assert_eq!(tile.units(), 0);
    assert!(!tile.is_dirty());

    // set attributes to the tile and make sure they are not updated, since the tile is a mountain
    tile.set_owner(Some(1));
    assert!(tile.owner().is_none());
    assert!(!tile.is_dirty());

    tile.set_units(999);
    assert_eq!(tile.units(), 0);
    assert!(!tile.is_dirty());

    // visibility can be updated on mountains though
    tile.reveal_to(9);
    assert!(tile.is_visible_by(9));
    assert!(tile.is_dirty());
    tile.hide_from(9);
    assert!(!tile.is_visible_by(9));
    assert!(tile.is_dirty());
    tile.set_clean();
    assert!(!tile.is_dirty());

    // now turn the tile into a normal tile and redo the same, this time making sure attributes
    // are updated.
    tile.make_open();
    // the tile should not be dirty since no-one sees it
    assert!(!tile.is_dirty());
    assert!(tile.owner().is_none());
    assert!(!tile.is_mountain());
    assert!(tile.is_open());
    assert!(!tile.is_general());
    assert!(!tile.is_city());
    assert_eq!(tile.units(), 0);

    tile.set_owner(Some(1));
    assert_eq!(tile.owner(), Some(1));
    check_dirty_and_clean(&mut tile);

    tile.set_owner(None);
    assert!(tile.owner().is_none());
    check_dirty_and_clean(&mut tile);

    tile.set_units(98);
    assert_eq!(tile.units(), 98);
    check_dirty_and_clean(&mut tile);

    tile.reveal_to(1);
    tile.reveal_to(2);
    tile.reveal_to(3);
    check_dirty_and_clean(&mut tile);
    assert!(tile.is_visible_by(1));
    assert!(tile.is_visible_by(2));
    assert!(tile.is_visible_by(3));
    assert!(!tile.is_visible_by(4));
    tile.hide_from(2);
    assert!(tile.is_visible_by(1));
    assert!(!tile.is_visible_by(2));
    assert!(tile.is_visible_by(3));
    tile.hide_from(1);
    tile.hide_from(3);
    tile.hide_from(3);
    assert!(!tile.is_visible_by(1));
    assert!(!tile.is_visible_by(2));
    assert!(!tile.is_visible_by(3));
    check_dirty_and_clean(&mut tile);
}

#[test]
fn tile_serialize() {
    let mut tile = Tile::new();

    let serialized = serde_json::to_string(&tile).unwrap();
    assert_eq!(serialized, r#"{"kind":"mountain"}"#);

    tile.make_open();
    let serialized = serde_json::to_string(&tile).unwrap();
    assert_eq!(serialized, r#"{}"#);

    tile.set_owner(Some(1));
    let serialized = serde_json::to_string(&tile).unwrap();
    assert_eq!(serialized, r#"{"owner":1}"#);

    tile.set_units(42);
    let serialized = serde_json::to_string(&tile).unwrap();
    assert_eq!(serialized, r#"{"owner":1,"units":42}"#);

    tile.make_general();
    let serialized = serde_json::to_string(&tile).unwrap();
    assert_eq!(serialized, r#"{"owner":1,"units":42,"kind":"general"}"#);

    tile.make_city();
    let serialized = serde_json::to_string(&tile).unwrap();
    assert_eq!(serialized, r#"{"owner":1,"units":42,"kind":"city"}"#);

    tile.reveal_to(1);
    let serialized = serde_json::to_string(&tile).unwrap();
    assert_eq!(serialized, r#"{"owner":1,"units":42,"kind":"city"}"#);
}

#[test]
fn conquer_occupied_tile_1() {
    let mut src = Tile::new();
    src.make_open();
    src.set_owner(Some(1));
    src.set_units(5);

    let mut dst = Tile::new();
    dst.make_open();
    dst.set_owner(Some(2));
    dst.set_units(2);

    let outcome = src.attack(&mut dst).unwrap();
    assert_eq!(outcome, MoveOutcome::TileCaptured(Some(2)));
    assert_eq!(src.units(), 1);
    assert_eq!(dst.units(), 2);
    assert_eq!(dst.owner(), Some(1));
}

#[test]
fn conquer_occupied_tile_2() {
    let mut src = Tile::new();
    src.make_open();
    src.set_owner(Some(1));
    src.set_units(6);

    let mut dst = Tile::new();
    dst.make_city();
    dst.set_owner(Some(2));
    dst.set_units(2);

    let outcome = src.attack(&mut dst).unwrap();
    assert_eq!(outcome, MoveOutcome::TileCaptured(Some(2)));
    assert_eq!(src.units(), 1);
    assert_eq!(dst.units(), 3);
    assert_eq!(dst.owner(), Some(1));
}

#[test]
fn conquer_unoccupied_tile() {
    let mut src = Tile::new();
    src.make_open();
    src.set_owner(Some(1));
    src.set_units(6);

    let mut dst = Tile::new();
    dst.make_open();

    let outcome = src.attack(&mut dst).unwrap();
    assert_eq!(outcome, MoveOutcome::TileCaptured(None));
    assert_eq!(src.units(), 1);
    assert_eq!(dst.units(), 5);
    assert_eq!(dst.owner(), Some(1));
}

#[test]
fn conquer_occupied_tile_fail() {
    let mut src = Tile::new();
    src.make_open();
    src.set_owner(Some(1));
    src.set_units(3);

    let mut dst = Tile::new();
    dst.make_open();
    dst.set_owner(Some(2));
    dst.set_units(2);

    let outcome = src.attack(&mut dst).unwrap();
    assert_eq!(outcome, MoveOutcome::StatuQuo);
    assert_eq!(src.units(), 1);
    assert_eq!(dst.units(), 0);
    assert_eq!(dst.owner(), Some(2));
}

#[test]
fn conquer_unoccupied_city_fail() {
    let mut src = Tile::new();
    src.make_open();
    src.set_owner(Some(1));
    src.set_units(10);

    let mut dst = Tile::new();
    dst.make_city();
    dst.set_owner(None);
    dst.set_units(9);

    let outcome = src.attack(&mut dst).unwrap();
    assert_eq!(outcome, MoveOutcome::StatuQuo);
    assert_eq!(src.units(), 1);
    assert_eq!(dst.units(), 0);
    assert_eq!(dst.owner(), None);
}

#[test]
fn conquer_city() {
    let mut src = Tile::new();
    src.make_open();
    src.set_owner(Some(1));
    src.set_units(10);

    let mut dst = Tile::new();
    dst.make_city();
    dst.set_owner(None);
    dst.set_units(7);

    let outcome = src.attack(&mut dst).unwrap();
    assert_eq!(outcome, MoveOutcome::TileCaptured(None));
    assert_eq!(src.units(), 1);
    assert_eq!(dst.units(), 2);
    assert_eq!(dst.owner(), Some(1));
}

#[test]
fn invalid_moves() {
    // source tile is a mountain
    let mut src = Tile::new();

    let mut dst = Tile::new();
    dst.make_city();
    dst.set_owner(None);
    dst.set_units(7);

    let outcome = src.attack(&mut dst);
    assert_eq!(outcome, Err(InvalidMove::FromInvalidTile));

    // source tile is open but has no owner
    src.make_open();
    src.set_units(10);

    let outcome = src.attack(&mut dst);
    assert_eq!(outcome, Err(InvalidMove::SourceTileNotOwned));

    // source tile is open and has an owner but has not enough unit
    src.make_open();
    src.set_owner(Some(1));
    src.set_units(1);

    let outcome = src.attack(&mut dst);
    assert_eq!(outcome, Err(InvalidMove::NotEnoughUnits));

    // source tile is now valid but dest tile is a mountain
    src.set_units(9);
    let mut dst_mountain = Tile::new(); // we don't have a `make_mountain()` method
    let outcome = src.attack(&mut dst_mountain);
    assert_eq!(outcome, Err(InvalidMove::ToInvalidTile));
}
