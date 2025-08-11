use crate::Name;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::{Registry, Unit};
use std::ops::Deref;
use std::sync::atomic::AtomicU64;
use std::sync::LazyLock;


#[derive(Copy, Clone, Hash, Debug, Default, Ord, PartialOrd, Eq, PartialEq, EncodeLabelSet)]
struct SourceLabel {
    source: Name
}


macro_rules! src {
    ($name:expr) => {
        SourceLabel {
            source: $name
        }
    };
}


macro_rules! metric {
    ($name:ident, $t:ty) => {
        static $name: LazyLock<$t> = LazyLock::new(Default::default);
    };
}


metric!(MAPPING_ERRORS, Family<SourceLabel, Counter>);
metric!(UNPARSED_TRANSACTION_ERRORS, Counter);
metric!(DATA_SOURCE_ERRORS, Family<SourceLabel, Counter>);
metric!(TRANSACTIONS_PUBLISHED, Family<SourceLabel, Counter>);
metric!(BLOCKS_PUBLISHED, Family<SourceLabel, Counter>);
metric!(LAST_BLOCK, Gauge<u64, AtomicU64>);
metric!(LAST_BLOCK_TIMESTAMP, Gauge);
metric!(ACTIVE_SUBSCRIPTIONS, Gauge);


pub fn register_mapping_error(source: Name) {
    MAPPING_ERRORS.get_or_create(&src!(source)).inc();
}


pub fn register_unparsed_transaction_error() {
    UNPARSED_TRANSACTION_ERRORS.inc();
}


pub fn register_data_source_error(source: Name) {
    DATA_SOURCE_ERRORS.get_or_create(&src!(source)).inc();
}


pub fn register_tx_publication(source: Name) {
    TRANSACTIONS_PUBLISHED.get_or_create(&src!(source)).inc();
}


pub fn register_block_publication(source: Name, slot: u64, timestamp: i64) {
    BLOCKS_PUBLISHED.get_or_create(&src!(source)).inc();
    LAST_BLOCK.set(slot);
    LAST_BLOCK_TIMESTAMP.set(timestamp);
}


pub fn register_subscription_scope() -> impl Drop {
    ACTIVE_SUBSCRIPTIONS.inc();
    SubscriptionGuard
}


struct SubscriptionGuard;
impl Drop for SubscriptionGuard {
    fn drop(&mut self) {
        ACTIVE_SUBSCRIPTIONS.dec();
    }
}


pub fn create_metrics_registry() -> Registry {
    let mut reg = Registry::default();

    reg.register(
        "spray_mapping_errors",
        "Number of data mapping errors",
        MAPPING_ERRORS.deref().clone()
    );
    
    reg.register(
        "spray_unparsed_transaction_errors",
        "Number of transactions with error deserialization failures",
        UNPARSED_TRANSACTION_ERRORS.deref().clone()
    );

    reg.register(
        "spray_data_source_errors",
        "Number of data source (connection) errors",
        DATA_SOURCE_ERRORS.deref().clone()
    );

    reg.register(
        "spray_transactions_published",
        "Number of transactions pushed to subscriptions",
        TRANSACTIONS_PUBLISHED.deref().clone()
    );

    reg.register(
        "spray_blocks_published",
        "Number of blocks pushed to subscriptions",
        BLOCKS_PUBLISHED.deref().clone()
    );
    
    reg.register(
        "spray_last_block",
        "Last published block",
        LAST_BLOCK.deref().clone()
    );
    
    reg.register_with_unit(
        "spray_last_block_timestamp",
        "Timestamp of the last published block",
        Unit::Seconds,
        LAST_BLOCK_TIMESTAMP.deref().clone()
    );
    
    reg.register(
        "spray_active_subscriptions",
        "Number of active client subscriptions",
        ACTIVE_SUBSCRIPTIONS.deref().clone()
    );

    reg
}