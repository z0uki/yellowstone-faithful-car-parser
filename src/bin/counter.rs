use {
    anyhow::Context,
    clap::Parser,
    indicatif::{MultiProgress, ProgressBar, ProgressStyle},
    prost::Message,
    solana_sdk::transaction::VersionedTransaction,
    solana_storage_proto::convert::generated,
    solana_transaction_status::TransactionStatusMeta,
    tokio::{fs::File, io::BufReader},
    yellowstone_faithful_car_parser::node::{Node, NodeReader, Nodes},
};

#[derive(Debug, Parser)]
#[clap(author, version, about = "count nodes in CAR files")]
struct Args {
    /// Path to CAR file
    #[clap(long)]
    pub car: String,

    /// Parse Nodes from CAR file
    #[clap(long)]
    pub parse: bool,

    /// Decode Nodes to Solana structs
    #[clap(long)]
    pub decode: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file = File::open(args.car)
        .await
        .context("failed to open CAR file")?;
    let mut reader = NodeReader::new(BufReader::new(file));

    if !args.parse {
        let bar = ProgressBar::no_length()
            .with_style(ProgressStyle::with_template("{spinner} {pos}").expect("valid template"));
        while reader.read_node().await?.is_some() {
            bar.inc(1);
        }
        bar.finish();
        return Ok(());
    }

    let mut bar = ReaderProgressBar::new();
    loop {
        let nodes = Nodes::read_until_block(&mut reader).await?;
        if nodes.nodes.is_empty() {
            break;
        }

        for node in nodes.nodes.values() {
            match node {
                Node::Transaction(frame) => {
                    bar.transaction += 1;
                    if !args.decode {
                        continue;
                    }

                    let _tx = bincode::deserialize::<VersionedTransaction>(&frame.data.data)
                        .context("failed to parse tx")?;

                    let buffer = nodes
                        .reassemble_dataframes(&frame.metadata)
                        .context("failed to build tx metadata")?;
                    let buffer = zstd::decode_all(buffer.as_slice())
                        .context("failed to decompress tx metadata")?; // TODO: failed to decompress tx metadata
                    let metadata = generated::TransactionStatusMeta::decode(buffer.as_slice())
                        .context("failed to decode tx metadata")?;
                    let _metadata = TransactionStatusMeta::try_from(metadata)
                        .context("failed to convert protobuf tx metadata")?;

                    bar.transaction_decode += 1;
                }
                Node::Entry(_) => bar.entry += 1,
                Node::Block(_) => bar.block += 1,
                Node::Subset(_) => bar.subset += 1,
                Node::Epoch(_) => bar.epoch += 1,
                Node::Rewards(frame) => {
                    bar.rewards += 1;
                    if !args.decode {
                        continue;
                    }

                    let buffer = nodes
                        .reassemble_dataframes(&frame.data)
                        .context("failed to build rewards")?;
                    let buffer = zstd::decode_all(buffer.as_slice())
                        .context("failed to decompress rewards")?;
                    let _rewards = generated::Rewards::decode(buffer.as_slice())
                        .context("failed to decode rewards")?;

                    bar.rewards_decode += 1;
                }
                Node::DataFrame(_) => bar.dataframe += 1,
            }
        }

        bar.report();
    }
    bar.finish();

    Ok(())
}

struct ReaderProgressBar {
    transaction: u64,
    pb_transaction: ProgressBar,
    entry: u64,
    pb_entry: ProgressBar,
    block: u64,
    pb_block: ProgressBar,
    subset: u64,
    pb_subset: ProgressBar,
    epoch: u64,
    pb_epoch: ProgressBar,
    rewards: u64,
    pb_rewards: ProgressBar,
    dataframe: u64,
    pb_dataframe: ProgressBar,
    //
    transaction_decode: u64,
    pb_transaction_decode: ProgressBar,
    rewards_decode: u64,
    pb_rewards_decode: ProgressBar,
}

impl ReaderProgressBar {
    fn new() -> Self {
        let multi = MultiProgress::new();
        Self {
            transaction: 0,
            pb_transaction: Self::create_pbbar(&multi, "parsed", "transaction"),
            entry: 0,
            pb_entry: Self::create_pbbar(&multi, "parsed", "entry"),
            block: 0,
            pb_block: Self::create_pbbar(&multi, "parsed", "block"),
            subset: 0,
            pb_subset: Self::create_pbbar(&multi, "parsed", "subset"),
            epoch: 0,
            pb_epoch: Self::create_pbbar(&multi, "parsed", "epoch"),
            rewards: 0,
            pb_rewards: Self::create_pbbar(&multi, "parsed", "rewards"),
            dataframe: 0,
            pb_dataframe: Self::create_pbbar(&multi, "parsed", "dataframe"),
            //
            transaction_decode: 0,
            pb_transaction_decode: Self::create_pbbar(&multi, "decoded", "transaction"),
            rewards_decode: 0,
            pb_rewards_decode: Self::create_pbbar(&multi, "decoded", "rewards"),
        }
    }

    fn create_pbbar(pb: &MultiProgress, kind1: &str, kind2: &str) -> ProgressBar {
        let pb = pb.add(ProgressBar::no_length());
        pb.set_style(
            ProgressStyle::with_template(&format!("{{spinner}} {kind1}:{kind2} {{pos}}"))
                .expect("valid template"),
        );
        pb
    }

    fn report(&self) {
        self.pb_transaction.set_position(self.transaction);
        self.pb_entry.set_position(self.entry);
        self.pb_block.set_position(self.block);
        self.pb_subset.set_position(self.subset);
        self.pb_epoch.set_position(self.epoch);
        self.pb_rewards.set_position(self.rewards);
        self.pb_dataframe.set_position(self.dataframe);
        self.pb_transaction_decode
            .set_position(self.transaction_decode);
        self.pb_rewards_decode.set_position(self.rewards_decode);
    }

    fn finish(&self) {
        self.pb_transaction.finish();
        self.pb_entry.finish();
        self.pb_block.finish();
        self.pb_subset.finish();
        self.pb_epoch.finish();
        self.pb_rewards.finish();
        self.pb_dataframe.finish();
        self.pb_transaction_decode.finish();
        self.pb_rewards_decode.finish();
    }
}
