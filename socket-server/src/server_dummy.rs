#![allow(dead_code)]

//use super::*;
//
//use std::cell::RefCell;
//use std::rc::Rc;
//
//pub struct ServerDummy {
//    connected: bool,
//    // printed to user
//    outputs: Rc<RefCell<Vec<String>>>,
//    // received by user
//    inputs: Rc<RefCell<Vec<String>>>,
//}
//
//impl ServerDummy {
//    pub fn new() -> Self {
//        ServerDummy {
//            connected: false,
//            outputs: Rc::new(RefCell::new(vec![])),
//            inputs: Rc::new(RefCell::new(vec![])),
//        }
//    }
//
//    pub fn get_outputs_pointer(&self) -> Rc<RefCell<Vec<String>>> {
//        self.container.outputs.clone()
//    }
//
//    pub fn get_inputs_pointer(&self) -> Rc<RefCell<Vec<String>>> {
//        self.inputs.clone()
//    }
//}
//
//impl Server for ServerDummy {
//    fn append_output(&mut self, pending_outputs: Vec<ServerOutput>) {
//        self.container.outputs.borrow_mut().extend(pending_outputs.into_iter().map(|i| i.output));
//    }
//
//    fn run(&mut self) -> ServerChanges {
//        let connection_id = ConnectionId(0);
//
//        // TODO: validate only connnections to 0
//        // if pending_outputs.iter().find(|i| i.dest_connections_id.)
////        let output_messages: Vec<String> = pending_outputs.into_iter().map(|i| i.output).collect();
////        self.container.outputs.borrow_mut().extend(output_messages);
//
//        let connects: Vec<ConnectionId> =
//            if self.connected {
//                vec![]
//            } else {
//                self.connected = true;
//                vec![connection_id]
//            };
//
//        let inputs: Vec<(ConnectionId, String)> =
//            self.inputs
//                .replace(vec![])
//                .into_iter()
//                .map(|msg| (connection_id, msg ))
//                .collect();
//
//        ServerChanges {
//            connects: connects,
//            disconnects: vec![],
//            inputs: inputs
//        }
//    }
//}
