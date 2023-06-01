use std::collections::HashMap;
use std::sync::mpsc::{Receiver, SyncSender};
pub struct  ChannelBarrierError;

#[derive(Debug)]
pub struct ChannelBarrier {
    senders : HashMap< u8, SyncSender<bool>>,
    receiver : Receiver<bool>,
    sender : SyncSender<bool>
}

impl ChannelBarrier {
    
    pub fn new(receiver : Receiver<bool> ,  sender : SyncSender<bool>) -> Self
    {
        Self{
            senders: HashMap::<u8, SyncSender<bool>>::new(),
            receiver,
            sender
        }
    }

    pub fn add_sender(& mut self , sender : SyncSender<bool> , tid : & u8 ) -> Result<() , ChannelBarrierError>
    {
        let result = self.senders.insert(tid.clone(), sender);

        return if result.is_some() {
            Ok(())
        } else {
            Err(ChannelBarrierError)
        }
    }

    pub fn get_sender(&mut self) -> Option<SyncSender<bool>>
    {
        Some(self.sender.clone())
    }

    pub fn wait(&mut self) -> ()
    {
        /*
            Notifica a tutti i receiver iscritti l'avvenuto superatamento della barriera
        */
        for sender in self.senders.iter()
        {
            sender.1.send(true);
        }

        for i in 0..self.senders.len()
        {
            let  val = self.receiver.recv()
                .expect("Impossibile leggere canale monodirezionale.");
            assert!(val);
        }
    }

}


