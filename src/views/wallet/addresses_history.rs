use crate::prelude::*;

use std::collections::{BTreeMap, HashMap};

use btc_heritage_wallet::bitcoin::{Amount, OutPoint};

use crate::{
    components::{
        balance::UIBtcAmount, heritage_configuration::UIExpirationBadge, timestamp::UITimestamp,
    },
    utils::{CCStr, CheapClone},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UIBalanceCellType {
    Utxo,
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct UIBalanceCell {
    cell_type: UIBalanceCellType,
    balance: UIBtcAmount,
}
impl LoadedElement for UIBalanceCell {
    type Loader = SkeletonLoader;

    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        let bg_color = match self.cell_type {
            UIBalanceCellType::Utxo => "bg-info/10",
            UIBalanceCellType::Incoming => "bg-success/10",
            UIBalanceCellType::Outgoing => "bg-error/10",
        };
        let title_color = match self.cell_type {
            UIBalanceCellType::Utxo => "text-info",
            UIBalanceCellType::Incoming => "text-success",
            UIBalanceCellType::Outgoing => "text-error",
        };
        let title = match self.cell_type {
            UIBalanceCellType::Utxo => "UTXO",
            UIBalanceCellType::Incoming => "Received",
            UIBalanceCellType::Outgoing => "Spent",
        };
        rsx! {
            div { class: "{bg_color} p-3 rounded-lg min-w-40 ",
                div { class: "text-sm font-semibold {title_color}", {title} }
                div { class: "text-xl font-bold text-center my-2",
                    LoadedComponent { input: m.map(self.balance) }
                }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            cell_type: UIBalanceCellType::Utxo,
            balance: UIBtcAmount::place_holder(),
        }
    }
}
impl UIBalanceCell {
    fn new(cell_type: UIBalanceCellType, amount: Amount) -> Self {
        Self {
            cell_type,
            balance: UIBtcAmount::from(amount),
        }
    }
}
impl From<TxIO> for UIBalanceCell {
    fn from(value: TxIO) -> Self {
        match value {
            TxIO::Incoming(tx_ioinner) => Self::new(UIBalanceCellType::Incoming, tx_ioinner.amount),
            TxIO::Outgoing(tx_ioinner) => Self::new(UIBalanceCellType::Outgoing, tx_ioinner.amount),
        }
    }
}
impl From<SimpleUtxo> for UIBalanceCell {
    fn from(value: SimpleUtxo) -> Self {
        Self::new(UIBalanceCellType::Utxo, value.amount)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UITxLineCell {
    top_connect: bool,
    balance_cell: Display<UIBalanceCell>,
    bottom_connect: bool,
}
impl LoadedElement for UITxLineCell {
    type Loader = SkeletonLoader;

    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "h-full w-full flex flex-col items-center",
                div { class: "basis-1/2 flex flex-row justify-center min-h-4",
                    if self.top_connect {
                        hr { class: "h-full w-1 bg-base-content" }
                    }
                }
                LoadedComponent { input: m.map(self.balance_cell) }
                div { class: "basis-1/2 flex flex-row justify-center min-h-4",
                    if self.bottom_connect {
                        hr { class: "h-full w-1 bg-base-content" }
                    }
                }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            top_connect: true,
            balance_cell: Display::Show(UIBalanceCell::place_holder()),
            bottom_connect: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UITxLine {
    tx_id: CCStr,
    tx_date: UITimestamp,
    tx_block: CCStr,
    cells: CheapClone<[UITxLineCell]>,
}
impl LoadedElement for UITxLine {
    type Loader = SkeletonLoader;

    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "overflow-clip py-2 my-auto flex flex-col gap-2",
                div { class: "flex flex-row gap-4",
                    div { class: "text-sm text-nowrap",
                        div { class: "font-semibold text-xs text-(--color-base-content)/60",
                            "Date"
                        }
                        div {
                            LoadedComponent { input: m.map(self.tx_date) }
                        }
                    }
                    div { class: "text-sm text-nowrap",
                        div { class: "font-semibold text-xs text-(--color-base-content)/60",
                            "Block"
                        }
                        div { {self.tx_block} }
                    }
                }
                div {
                    div { class: "font-semibold text-xs text-(--color-base-content)/60",
                        "Tx Id"
                    }
                    div { class: "text-sm font-mono wrap-break-word", {self.tx_id} }
                }
            }
            LoadedComponent::<CheapClone<[UITxLineCell]>> { input: m.map(self.cells) }
            div { class: "col-span-full border-b" }
        }
    }

    fn place_holder() -> Self {
        Self {
            tx_id: CCStr::from("de3f6e7897333b5a4067237ba10c6122e6480ae1fc7372e7117dbfe5048e942f"),
            tx_date: UITimestamp::place_holder(),
            tx_block: CCStr::place_holder(),
            cells: CheapClone::from_iter([
                UITxLineCell::place_holder(),
                UITxLineCell::place_holder(),
            ]),
        }
    }
}

struct OutPointColumnPlacement {
    free_cols: Vec<usize>,
    pending_free: Vec<usize>,
    max_cols: usize,
    utxo_col: HashMap<OutPoint, usize>,
}
impl OutPointColumnPlacement {
    fn new() -> Self {
        Self {
            free_cols: Vec::new(),
            pending_free: Vec::new(),
            max_cols: 0,
            utxo_col: HashMap::new(),
        }
    }
    fn change_row(&mut self) {
        self.free_cols.append(&mut self.pending_free);
    }

    fn is_col_free(&self, col: usize) -> bool {
        self.free_cols.contains(&col)
    }
    fn get_col(&mut self, outpoint: OutPoint) -> usize {
        // We will always see every outpoint two times
        // If it is the first time we see the OutPoint, that's because the request is for
        // it's top of line box (UTXO or Outgoing)
        // If OutPoint already has a col, that's because the request is for
        // it's bottom of line box (Incoming)
        if let Some(col) = self.utxo_col.remove(&outpoint) {
            // OutPoint already has a col
            // It will be freed for next row
            self.pending_free.push(col);
            col
        } else {
            // First time we see the OutPoint
            // Need to attribute a col to it
            let col = if let Some(col) = self.free_cols.pop() {
                // A free col is available, use it
                col
            } else {
                // No free col, need to add a new one
                let col = self.max_cols;
                self.max_cols += 1;
                col
            };
            // Register the outpoint column
            self.utxo_col.insert(outpoint, col);
            col
        }
    }
}

/// Transaction statistics display component for expanded address details
#[derive(Debug, Clone, PartialEq)]
struct UIAddressTransactionsHistoryDetail {
    grid_cols: usize,
    utxo_cells: CheapClone<[UITxLineCell]>,
    tx_lines: CheapClone<[UITxLine]>,
}
impl LoadedElement for UIAddressTransactionsHistoryDetail {
    type Loader = TransparentLoader;

    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        let grid_cols = match self.grid_cols {
            1 => "grid-cols-[calc(var(--spacing)*68)_repeat(1,1fr)]",
            2 => "grid-cols-[calc(var(--spacing)*68)_repeat(2,1fr)]",
            3 => "grid-cols-[calc(var(--spacing)*68)_repeat(3,1fr)]",
            4 => "grid-cols-[calc(var(--spacing)*68)_repeat(4,1fr)]",
            5 => "grid-cols-[calc(var(--spacing)*68)_repeat(5,1fr)]",
            6 => "grid-cols-[calc(var(--spacing)*68)_repeat(6,1fr)]",
            7 => "grid-cols-[calc(var(--spacing)*68)_repeat(7,1fr)]",
            8 => "grid-cols-[calc(var(--spacing)*68)_repeat(8,1fr)]",
            9 => "grid-cols-[calc(var(--spacing)*68)_repeat(9,1fr)]",
            10 => "grid-cols-[calc(var(--spacing)*68)_repeat(10,1fr)]",
            11 => "grid-cols-[calc(var(--spacing)*68)_repeat(11,1fr)]",
            _ => "grid-cols-[calc(var(--spacing)*68)_repeat(12,1fr)]",
        };
        rsx! {
            div { class: "grid {grid_cols} gap-x-4",
                if self.utxo_cells.len() > 0 {
                    div { class: "my-auto",
                        div { class: "font-bold",
                            LoadedComponent { input: m.map(self.utxo_cells.len()) }
                            " UTXO"
                            if self.utxo_cells.len() > 1 {
                                "s"
                            }
                        }
                        div { "can be spent from this address" }
                    }
                    LoadedComponent::<CheapClone<[UITxLineCell]>> { input: m.map(self.utxo_cells) }
                    div { class: "col-span-full border-b" }
                }
                LoadedComponent::<CheapClone<[UITxLine]>> { input: m.map(self.tx_lines) }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            grid_cols: 1,
            utxo_cells: CheapClone::from_iter([UITxLineCell::place_holder()]),
            tx_lines: CheapClone::from_iter([UITxLine::place_holder()]),
        }
    }
}

impl From<(&[TransactionStats], &[SimpleUtxo])> for UIAddressTransactionsHistoryDetail {
    fn from((tx_stats, utxos): (&[TransactionStats], &[SimpleUtxo])) -> Self {
        log::debug!(
            "UIAddressTransactionsHistoryDetail::From<(&[TransactionStats], &[SimpleUtxo])>"
        );
        let mut outpoint_placement = OutPointColumnPlacement::new();

        let mut utxo_cells = Vec::new();
        for sutxo in utxos.iter() {
            // We don't care for the result, we know it will be 0, 1, 2...
            outpoint_placement.get_col(sutxo.outpoint);
            utxo_cells.push(UITxLineCell {
                top_connect: false,
                balance_cell: Show(UIBalanceCell::from(*sutxo)),
                bottom_connect: true,
            });
        }
        let mut tx_lines = Vec::new();
        for tx in tx_stats.iter() {
            // Signal the OutPointColumnPlacement that we changed row
            outpoint_placement.change_row();

            let tx_id = CCStr::from(tx.id.to_string());
            let (tx_date, tx_block) = tx
                .block_time
                .as_ref()
                .map(|bt| {
                    (
                        UITimestamp::new_full(bt.timestamp),
                        CCStr::from(format!("#{}", bt.height)),
                    )
                })
                .unwrap_or_else(|| (UITimestamp::none(), CCStr::from("-")));

            let mut tmp_cells_btree = BTreeMap::new();
            for tx_io in tx.in_out.iter() {
                let col = outpoint_placement.get_col(tx_io.outpoint);
                tmp_cells_btree.insert(
                    col,
                    UITxLineCell {
                        top_connect: matches!(tx_io, TxIO::Incoming(_)),
                        balance_cell: Show(UIBalanceCell::from(*tx_io)),
                        bottom_connect: matches!(tx_io, TxIO::Outgoing(_)),
                    },
                );
            }
            let mut cells = Vec::new();
            // Crucial: It works because the BTree always returns the "col" in ascending order
            for (col, cell) in tmp_cells_btree {
                // If the col is not the current size, then we pad with "connector cells"
                if cells.len() < col {
                    cells.extend((cells.len()..col).map(|col_to_pad| {
                        let draw_line = !outpoint_placement.is_col_free(col_to_pad);
                        UITxLineCell {
                            top_connect: draw_line,
                            balance_cell: Display::None,
                            bottom_connect: draw_line,
                        }
                    }));
                }
                cells.push(cell);
            }
            tx_lines.push(UITxLine {
                tx_id,
                tx_date,
                tx_block,
                cells: cells.into(),
            });
        }

        Self {
            grid_cols: outpoint_placement.max_cols,
            utxo_cells: utxo_cells.into(),
            tx_lines: tx_lines.into(),
        }
    }
}

/// Complete expandable row combining summary and detailed views
#[derive(Debug, Clone, PartialEq)]
struct UIAddressesHistoryExpandableRow {
    address: CCStr,
    address_origin: CCStr,
    expiration_badge: LResult<UIExpirationBadge>,
    balance: LResult<UIBtcAmount>,
    utxo_count: LResult<usize>,
    transaction_count: LResult<usize>,
    transaction_detail: LResult<UIAddressTransactionsHistoryDetail>,
}

impl LoadedElement for UIAddressesHistoryExpandableRow {
    type Loader = TransparentLoader;

    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "collapse border-b border-base-content/5 rounded-none last:border-b-0 odd:bg-base-200",
                input { r#type: "checkbox", class: "!py-3 !min-h-0" }

                // Summary row (always visible)
                div { class: "collapse-title min-h-0 p-0",
                    div { class: "grid grid-cols-6 gap-1 px-4 py-3 items-center cursor-pointer",

                        // Address column
                        div { class: "col-span-3 font-mono truncate",
                            LoadedComponent { input: m.map(self.address.clone()) }
                        }

                        // Heritage config column
                        div { class: "flex justify-center",
                            LoadedComponent { input: m.lc_map(self.expiration_badge.clone().into()) }
                        }

                        // Transactions (UTXOs) column
                        div { class: "text-center",
                            LoadedComponent { input: m.lc_map(self.transaction_count.into()) }
                            " ("
                            LoadedComponent { input: m.lc_map(self.utxo_count.into()) }
                            ")"
                        }

                        // Balance column
                        div {
                            LoadedComponent { input: m.lc_map(self.balance.into()) }
                        }
                    }
                }

                // Expanded content (shown when collapsed)
                div { class: "collapse-content p-0",
                    div { class: "overflow-x-auto w-[calc(100vw-var(--spacing)*16)] bg-base-300",
                        div { class: "flex flex-row gap-4 p-6 bg-base-300 border-t border-base-content/10 w-max",

                            // Address details section
                            div { class: "max-w-[33vw]",
                                h4 { class: "font-semibold text-base text-primary",
                                    "Heritage Configuration Status"
                                }
                                div {
                                    LoadedComponent { input: m.lc_map(self.expiration_badge.clone().into()) }
                                    span { class: "ml-2 text-wrap wrap-normal",
                                        LoadedComponent { input: m.lc_map(self.expiration_badge.lrmap(|eb| eb.tooltip()).into()) }
                                    }
                                }
                                h4 { class: "font-semibold text-base text-primary",
                                    "Address Details"
                                }
                                div { class: "font-mono text-sm wrap-break-word",
                                    div {
                                        LoadedComponent { input: m.map(self.address_origin) }
                                    }
                                    div {
                                        LoadedComponent { input: m.map(self.address) }
                                    }
                                }
                            }
                            div {
                                h4 { class: "font-semibold text-base text-primary",
                                    "Transaction & UTXOs"
                                }
                                // Transaction details section
                                LoadedComponent { input: m.lc_map(self.transaction_detail.into()) }
                            }
                        }
                    }
                }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            address: CCStr::place_holder(),
            address_origin: CCStr::place_holder(),
            expiration_badge: None,
            balance: None,
            utxo_count: None,
            transaction_count: None,
            transaction_detail: None,
        }
    }
}

impl LoadedSuccessConversionMarker
    for TypeCouple<WalletAddressWithInfo, UIAddressesHistoryExpandableRow>
{
}
impl FromRef<WalletAddressWithInfo> for UIAddressesHistoryExpandableRow {
    fn from_ref(wallet_address_with_info: &WalletAddressWithInfo) -> Self {
        let WalletAddressWithInfo {
            wallet_address,
            heritage_config_infos,
            tx_stats,
            utxo_stats,
        } = wallet_address_with_info;

        let address = CCStr::from(wallet_address.address().to_string());
        let (fg, dp) = wallet_address.origin();
        let address_origin = CCStr::from(format!("[{fg}/{dp}]"));

        let expiration_badge = heritage_config_infos.lrmap(|heritage_config_infos| {
            UIExpirationBadge::from((
                heritage_config_infos.expiration_status,
                match utxo_stats {
                    Some(Ok(utxo_stats)) if utxo_stats.balance != Amount::ZERO => true,
                    _ => false,
                },
            ))
        });

        let balance = utxo_stats.lrmap(|utxo_stats| UIBtcAmount::from(utxo_stats.balance));
        let utxo_count = utxo_stats.lrmap(|utxo_stats| utxo_stats.utxos.len());

        let transaction_count = tx_stats.lrmap(|tx_stats| tx_stats.len());

        let transaction_detail =
            if let (Some(Ok(tx_stats)), Some(Ok(utxo_stats))) = (tx_stats, utxo_stats) {
                Some(Ok(UIAddressTransactionsHistoryDetail::from((
                    tx_stats.as_ref(),
                    utxo_stats.utxos.as_ref(),
                ))))
            } else {
                None
            };

        Self {
            address,
            address_origin,
            expiration_badge,
            balance,
            utxo_count,
            transaction_count,
            transaction_detail,
        }
    }
}

/// Main component for displaying wallet addresses with expandable details
#[component]
pub(super) fn AddressesHistory() -> Element {
    log::debug!("AddressesHistory Rendered");

    let mut filter_addresses_without_tx = use_signal(|| true);
    let wallet_addresses_infos = use_context::<FMemo<CheapClone<[WalletAddressWithInfo]>>>();
    let filtered_wallet_addresses_infos = use_memo(move || {
        let active_filter = filter_addresses_without_tx();
        wallet_addresses_infos.lrmap(|addresses| {
            addresses
                .iter()
                .filter(|addr| {
                    !active_filter
                        || match addr.tx_stats {
                            Some(Ok(ref stats)) => !stats.is_empty(),
                            _ => false,
                        }
                })
                .cloned()
                .collect::<CheapClone<[_]>>()
        })
    });

    use_drop(|| log::debug!("AddressesHistory Dropped"));

    rsx! {
        div { class: "max-h-[calc(100vh-var(--spacing)*32)] overflow-y-auto rounded-box border border-base-content/5 shadow-md bg-base-100 my-4",
            // Title
            h2 { class: "text-2xl font-bold p-4", "Wallet Addresses" }

            // Header row
            div { class: "sticky top-0 z-10 bg-base-100 grid grid-cols-6 gap-1 items-center p-4 font-semibold text-base text-(--color-base-content)/60 border-b border-base-content/10",
                div { class: "col-span-3 flex flex-row gap-8 items-center",
                    "Address"
                    // Filter toggle
                    label { class: "label text-wrap",
                        input {
                            r#type: "checkbox",
                            class: "toggle toggle-secondary",
                            checked: filter_addresses_without_tx(),
                            onchange: move |evt| filter_addresses_without_tx.set(evt.checked()),
                        }
                        span { class: "text-base ml-2",
                            if filter_addresses_without_tx() {
                                "Only used addresses"
                            } else {
                                "All generated addresses"
                            }
                        }
                    }
                }
                div { class: "text-center", "Heritage Config" }
                div { class: "text-center", "Transactions (UTXOs)" }
                div { "Balance" }
            }

            // Expandable address rows
            LoadedComponent::<CheapClone<[UIAddressesHistoryExpandableRow]>> { input: filtered_wallet_addresses_infos.into() }
        
        }
    }
}
