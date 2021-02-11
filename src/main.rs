mod lib;

use std::str::FromStr;

use clap::Clap;
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use ton_block::{MsgAddressInt, Serializable};
use ton_types::Result;

use collector_lib::CollectorMessageParams;

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
                let message = collector_lib::create_message(CollectorMessageParams {
                    key: cmd.key.pair,
                    to: cmd.to,
                    init: cmd.init,
                    destroy: cmd.destroy,
                    seqno: cmd.seqno,
                    id: cmd.id,
                    ttl: cmd.ttl,
                })?;

                let serialized = ton_types::serialize_toc(&message.serialize()?)?;
                println!("{}", base64::encode(&serialized));
            }
            SubCommand::Addr(cmd) => {
                let address = collector_lib::compute_deposit_address(cmd.key.pair, cmd.id)?;
                println!("{}", address);
            }
        }

        Ok(())
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
    #[clap(long, about = "Whether to destroy contract after this message")]
    destroy: bool,
    #[clap(
        long,
        about = "Message sequence number",
        conflicts_with = "init",
        default_value = "0"
    )]
    seqno: u32,
    #[clap(long, about = "Wallet id", default_value = "0")]
    id: u32,
    #[clap(long, about = "Message timeout in seconds", default_value = "60")]
    ttl: u32,
}

#[derive(Clap)]
struct CmdGenerateAddress {
    key: InputKey,
    #[clap(long, about = "Wallet id", default_value = "0")]
    id: u32,
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
