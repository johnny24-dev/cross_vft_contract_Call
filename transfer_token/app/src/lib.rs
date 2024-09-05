#![no_std]
pub mod services;
pub mod clients;

use sails_rs::{
    prelude::*,
    gstd::{
        calls::GStdRemoting,
        msg
    }
};
use services::transfers_service::TransferService;
use clients::extended_new_vft::Vft as VftClient;

#[derive(Default)]
pub struct TransferProgram;

#[program]
impl TransferProgram {

    pub fn new() -> Self {
        Self
    }

    #[route("TransferCall")]
    pub fn transfer_service(&self) -> TransferService<VftClient<GStdRemoting>> {
        let vft_client = VftClient::new(GStdRemoting);
        TransferService::new(vft_client)
    }
}
