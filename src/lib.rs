use ed25519_dalek::ed25519::signature::Signature;
use ed25519_dalek::{Keypair, Signer};
use ton_block::{
    ExternalInboundMessageHeader, InternalMessageHeader, Message, MsgAddressInt, Serializable,
    StateInit,
};
use ton_types::{BuilderData, Cell, IBitstring, Result, SliceData, UInt256};

pub struct CollectorMessageParams {
    pub key: Keypair,
    pub to: MsgAddressInt,
    pub init: bool,
    pub destroy: bool,
    pub seqno: u32,
    pub id: u32,
    pub ttl: u32,
}

pub fn generate_message(params: CollectorMessageParams) -> Result<()> {
    // Generate wallet init data
    let init_data = InitData::from_key(&params.key).with_wallet_id(params.id);

    // Create external message
    let mut message = Message::with_ext_in_header(ExternalInboundMessageHeader {
        dst: init_data.compute_addr()?,
        ..Default::default()
    });

    if params.init {
        // Attach state_init to deploy wallet
        message.set_state_init(init_data.make_state_init()?);
    }

    // Attach internal message
    let transfer_msg = make_gift_message(
        &init_data,
        &params.key,
        params.seqno,
        params.ttl,
        &[Gift {
            flags: 128 + if params.destroy { 32 } else { 0 },
            bounce: false,
            destination: params.to,
            amount: 0,
        }],
    )?;
    message.set_body(transfer_msg.into());

    // Serialize and with as base64
    let message = ton_types::serialize_toc(&message.serialize()?)?;
    println!("{}", base64::encode(&message));

    Ok(())
}

pub fn generate_address(key: Keypair, id: u32) -> Result<()> {
    let address = InitData::from_key(&key).with_wallet_id(id).compute_addr()?;
    println!("{}", address);
    Ok(())
}

/// WalletV3 init data
struct InitData {
    pub seqno: u32,
    pub wallet_id: u32,
    pub public_key: UInt256,
}

impl InitData {
    pub fn from_key(key: &Keypair) -> Self {
        Self {
            seqno: 0,
            wallet_id: 0,
            public_key: key.public.as_bytes().into(),
        }
    }

    pub fn with_wallet_id(mut self, id: u32) -> Self {
        self.wallet_id = id;
        self
    }

    pub fn compute_addr(&self) -> Result<MsgAddressInt> {
        let init_state = self.make_state_init()?.serialize()?;
        let hash = init_state.repr_hash();
        MsgAddressInt::with_standart(None, 0, hash.into())
    }

    pub fn make_state_init(&self) -> Result<StateInit> {
        Ok(StateInit {
            code: Some(load_code()),
            data: Some(self.serialize()?),
            ..Default::default()
        })
    }

    pub fn deserialize(data: Cell) -> Result<Self> {
        let mut slice: SliceData = data.into();
        let seqno = slice.get_next_u32()?;
        let wallet_id = slice.get_next_u32()?;
        let public_key = slice.get_next_bytes(32)?.into();

        Ok(InitData {
            seqno,
            wallet_id,
            public_key,
        })
    }

    pub fn serialize(&self) -> Result<Cell> {
        let mut data = BuilderData::new();
        data.append_u32(self.seqno)?
            .append_u32(self.wallet_id)?
            .append_raw(self.public_key.as_slice(), 256)?;
        data.into_cell()
    }
}

/// WalletV3 transfer info
struct Gift {
    pub flags: u8,
    pub bounce: bool,
    pub destination: MsgAddressInt,
    pub amount: u64,
}

fn make_gift_message(
    init_data: &InitData,
    key: &Keypair,
    seqno: u32,
    ttl: u32,
    gifts: &[Gift],
) -> Result<Cell> {
    if gifts.len() >= MAX_GIFT_COUNT {
        ton_types::fail!("too many gifts");
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut message = BuilderData::new();
    message
        .append_u32(init_data.wallet_id)?
        .append_u32(now as u32 + ttl)?
        .append_u32(seqno)?;

    for gift in gifts {
        let internal_message = Message::with_int_header(InternalMessageHeader {
            ihr_disabled: true,
            bounce: gift.bounce,
            dst: gift.destination.clone(),
            value: gift.amount.into(),
            ..Default::default()
        });

        message
            .append_u8(gift.flags)?
            .append_reference_cell(internal_message.serialize()?);
    }

    let signature = key.sign(message.clone().into_cell()?.repr_hash().as_slice());
    message.prepend_raw(signature.as_bytes(), ed25519_dalek::SIGNATURE_LENGTH * 8)?;

    message.into_cell()
}

fn load_code() -> Cell {
    ton_types::deserialize_tree_of_cells(&mut std::io::Cursor::new(WALLET_V3_CODE)).unwrap()
}

const MAX_GIFT_COUNT: usize = 4;
const WALLET_V3_CODE: &[u8] = include_bytes!("../contracts/wallet_code.boc");
