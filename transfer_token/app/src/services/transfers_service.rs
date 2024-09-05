use sails_rs::calls::{Call, Query};
use sails_rs::{
    prelude::*,
    gstd::msg,
};

use gstd::exec;

use crate::clients::extended_new_vft::traits::Vft;

pub struct TransferService<VftClient>{
    pub vft_client:VftClient
}
#[derive(Encode, TypeInfo, Decode)]
pub enum Events {
    Transfer {
        vft:ActorId,
        from:ActorId,
        to:ActorId,
        value:U256
    }
}

#[service(events = Events)]
impl<VftClient>TransferService<VftClient>
where VftClient: Vft {

    pub fn new(
        vft_client: VftClient
    ) -> Self {
        Self {
            vft_client
        }
    }

    pub async fn transfer(&mut self,token:ActorId, to:ActorId, value:U256) -> bool {
        let caller = msg::source();

        let user_balace_response = self.vft_client.balance_of(caller).recv(token).await;
        let Ok(user_balace) = user_balace_response else {
            return false;
        };
        if user_balace < value {
            return false;
        }

        let send_response = self.vft_client.transfer(to, value).send_recv(token).await;
        let Ok(transfer_status) = send_response else {
            return false;
        };
        if transfer_status {
            self.notify_on(Events::Transfer { vft: token, from: caller, to, value }).unwrap();
        };
        transfer_status
    }

    pub async fn get_balance_from_contract(&self, owner:ActorId, token:ActorId) -> U256 {
        let user_balace_response = self.vft_client.balance_of(owner).recv(token).await;
        let Ok(user_balace) = user_balace_response else {
            return U256::zero();
        };
        user_balace
    }


}