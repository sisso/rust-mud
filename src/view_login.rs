//fn ask_name(connection: &mut Connection) -> std::io::Result<String> {
//    loop {
//        let provided_name = connection.read_field("login: ")?;
//
//        if provided_name.len() <= 3 {
//            let msg = format!("invalid login name '{}' Try again.", provided_name);
//            connection.writeln(&msg)?;
//            continue;
//        }
//
//        connection.writeln(format!("nice name {}", &provided_name).as_str())?;
//        return Ok(provided_name);
//    }
//}
//
//pub fn handle_login(id: u32, mut connection: Connection) -> std::io::Result<PlayerConnection> {
//    let login: String = ask_name(&mut connection)?;
//    Ok(PlayerConnection {
//        id,
//        login,
//        connection
//    })
//}
