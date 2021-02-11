mod lib;

use std::str::FromStr;

use clap::Clap;
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use ton_block::MsgAddressInt;
use ton_types::Result;

fn main() {
    let args: Args = Args::parse();
    if let Err(e) = args.cmd.execute() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

impl SubCommand {
    pub fn execute(self) -> Result<()> {
        match self {
            SubCommand::Msg(cmd) => {
                collector_lib::generate_message(cmd.key.pair, cmd.to, cmd.init, cmd.seqno, cmd.ttl)
            }
            SubCommand::Addr(cmd) => collector_lib::generate_address(cmd.key.pair),
        }
    }
}

#[derive(Clap)]
#[clap(version = "1.0")]
struct Args {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(about = "Generate collector message")]
    Msg(CmdGenerateMessage),
    #[clap(about = "Generate target address")]
    Addr(CmdGenerateAddress),
}

#[derive(Clap)]
struct CmdGenerateMessage {
    key: InputKey,
    #[clap(long, about = "Destination address, where funds will be collected")]
    to: MsgAddressInt,
    #[clap(long, about = "Whether to attach init data to the message")]
    init: bool,
    #[clap(
        long,
        about = "Message sequence number",
        conflicts_with = "init",
        default_value = "0"
    )]
    seqno: u32,
    #[clap(long, about = "Message timeout in seconds", default_value = "60")]
    ttl: u32,
}

#[derive(Clap)]
struct CmdGenerateAddress {
    key: InputKey,
}

struct InputKey {
    pair: Keypair,
}

impl FromStr for InputKey {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self> {
        let data = hex::decode(s)?;
        let secret = SecretKey::from_bytes(&data)?;
        let public = PublicKey::from(&secret);
        Ok(Self {
            pair: Keypair { secret, public },
        })
    }
}
