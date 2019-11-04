enum Events {
    Moved {
        player_id: Option<PlayerId>,
        mob_id: MobId,
        from_room: RoomId,
        from_dir: Dir,
        to_dir: Dir,
        to_room: RoomId,
        mob_label: String,
    },

    MoveCanNot {
        player_id: PlayerId,
        dir: Dir,
    }
}
