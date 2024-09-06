use gstd::prog::ProgramGenerator;
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
    },
    VftCreated {
        vft:ActorId,
        name:String,
        symbol:String,
        decimals:u8
    }
}

#[derive(Debug, Decode, Encode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitConfig {
    pub name: String,
    pub symbol:String,
    pub decimals:u8
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

    pub async fn create_vft(&mut self, code_id:CodeId, name:String, symbol:String, decimals:u8) -> bool {
       let payload = InitConfig {
        name:name.clone(),
        symbol:symbol.clone(),
        decimals
       };
       let payload_bytes = ["New".encode(), payload.encode()].concat();
       let create_program_future =
            ProgramGenerator::create_program_bytes(
                code_id,
                payload_bytes,
                0,
            )
            .map_err(|e| false);

        let Ok((_, address)) = create_program_future else {
            return false;
        };
        self.notify_on(Events::VftCreated { vft: address, name, symbol, decimals }).unwrap();
        true
    }
       

    pub async fn transfer(&mut self,token:ActorId, to:ActorId, value:U256) -> bool {
        let caller = exec::program_id();

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

    pub async fn transfer_from(&mut self, token:ActorId, from:ActorId, to:ActorId, value:U256) -> bool {

        let user_balace_response = self.vft_client.balance_of(from).recv(token).await;
        let Ok(user_balace) = user_balace_response else {
            return false;
        };
        if user_balace < value {
            return false;
        }

        // check allowance
        let allowance_response = self.vft_client.allowance(from, exec::program_id()).recv(token).await;
        let Ok(allowance) = allowance_response else {
            return false;
        };
        if allowance < value {
            return false;
        }

        let send_response = self.vft_client.transfer_from(from, to, value).send_recv(token).await;
        let Ok(transfer_status) = send_response else {
            return false;
        };
        if transfer_status {
            self.notify_on(Events::Transfer { vft: token, from, to, value }).unwrap();
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