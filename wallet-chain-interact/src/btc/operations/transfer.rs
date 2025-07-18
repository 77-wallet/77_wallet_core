use crate::btc::{
    ParseBtcAddress,
    consts::{self, EXPEND_FEE_RATE},
    provider::Provider,
    signature::{self, MultisigSignParams},
    utxos::UtxoList,
};
use bitcoin::{Amount, TxIn, consensus, transaction::Version};
use wallet_types::chain::{self, address::r#type::BtcAddressType};
use wallet_utils::unit;

#[derive(Debug)]
pub struct TransferArg {
    pub from: bitcoin::Address,
    pub to: bitcoin::Address,
    pub value: bitcoin::Amount,
    pub change_address: bitcoin::Address,
    pub address_type: BtcAddressType,
    pub fee_rate: Option<u64>,
    pub spend_all: bool,
}

impl TransferArg {
    pub fn new(
        from: &str,
        to: &str,
        value: &str,
        address_type: Option<String>,
        network: chain::network::NetworkKind,
    ) -> crate::Result<Self> {
        let paras = ParseBtcAddress::new(network);

        let value = unit::convert_to_u256(value, consts::BTC_DECIMAL)?;
        let value = bitcoin::Amount::from_sat(value.to::<u64>());

        let address_type = BtcAddressType::try_from(address_type)?;
        Ok(Self {
            from: paras.parse_address(from)?,
            to: paras.parse_address(to)?,
            change_address: paras.parse_address(from)?,
            value,
            address_type,
            fee_rate: None,
            spend_all: false,
        })
    }

    pub fn with_spend_all(mut self, spend_all: bool) -> Self {
        self.spend_all = spend_all;
        self
    }

    /// unit is sat/vb
    pub async fn get_fee_rate(
        &self,
        provider: &Provider,
        network: wallet_types::chain::network::NetworkKind,
    ) -> crate::Result<bitcoin::Amount> {
        if let Some(fee_rate) = self.fee_rate {
            Ok(bitcoin::Amount::from_sat(fee_rate))
        } else {
            let fetched_fee_rate = provider
                .fetch_fee_rate(consts::FEE_RATE as u32, network)
                .await?;
            Ok(fetched_fee_rate)
        }
    }
}

impl TransferArg {
    pub fn build_transaction(&self, mut utxo: UtxoList) -> crate::Result<TransferBuilder> {
        let (input, output) = if self.spend_all {
            (utxo.selected_all()?, vec![])
        } else {
            (
                utxo.inputs_from_utxo(self.value)?,
                vec![bitcoin::TxOut {
                    value: self.value,
                    script_pubkey: self.to.script_pubkey(),
                }],
            )
        };

        let transaction = bitcoin::Transaction {
            version: Version(2),
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input,
            output,
        };

        Ok(TransferBuilder {
            transaction,
            utxo,
            multisig_sign_params: None,
        })
    }

    // 初始构建交易
    pub fn build_transaction_v1(&self, mut utxo: UtxoList) -> crate::Result<TransferBuilder> {
        // check balance
        let input = utxo.inputs_from_utxo(self.value)?;
        let output = vec![bitcoin::TxOut {
            value: self.value,
            script_pubkey: self.to.script_pubkey(),
        }];

        let transaction = bitcoin::Transaction {
            version: Version(2),
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input,
            output,
        };

        Ok(TransferBuilder {
            transaction,
            utxo,
            multisig_sign_params: None,
        })
    }

    // build transaction with fee  fee unit is Btc
    // include change
    pub fn build_with_fee(
        &self,
        mut utxo: UtxoList,
        fee: Amount,
    ) -> crate::Result<TransferBuilder> {
        let amount = self.value + fee;
        let input = utxo.inputs_from_utxo(amount)?;

        let mut output = vec![];
        let spend = bitcoin::TxOut {
            value: self.value,
            script_pubkey: self.to.script_pubkey(),
        };
        output.push(spend);

        // select utxo amount
        let total_input = utxo.total_input_amount();

        if total_input > amount {
            let change = total_input - amount;
            let change_output = bitcoin::TxOut {
                value: change,
                script_pubkey: self.change_address.script_pubkey(),
            };
            output.push(change_output);
        }

        let transaction = bitcoin::Transaction {
            version: Version(2),
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input,
            output,
        };

        Ok(TransferBuilder {
            transaction,
            utxo,
            multisig_sign_params: None,
        })
    }
}

pub struct TransferBuilder {
    pub transaction: bitcoin::Transaction,
    pub utxo: UtxoList,
    pub multisig_sign_params: Option<MultisigSignParams>,
}
impl TransferBuilder {
    pub fn set_multisig_params(&mut self, multisig_sign_params: MultisigSignParams) {
        self.multisig_sign_params = Some(multisig_sign_params)
    }
}

#[derive(Debug)]
pub enum CheckTransactionRes {
    // input < out
    Less(bitcoin::Amount),
    // input == out
    Equal,
    // input > out
    Greater(bitcoin::Amount),
}

// build
impl TransferBuilder {
    pub fn transactin_size(
        &mut self,
        fee_rate: Amount,
        params: &TransferArg,
    ) -> crate::Result<usize> {
        if params.spend_all {
            self.spent_all_set_fee(fee_rate, params.to.clone(), params.address_type)
        } else {
            // 找零和手续费配置
            self.change_and_fee(
                fee_rate,
                params.change_address.clone(),
                params.address_type,
                params.value,
            )
        }
    }

    // 校验结果是否正确
    // fn check_transaction(
    //     &self,
    //     fee: Amount,
    //     change_address: bitcoin::Address,
    //     change_amount: Amount,
    // ) -> CheckTransactionRes {
    //     let total_input = self.utxo.total_input_amount();

    //     // 不去包含找零的
    //     let total_output = self
    //         .transaction
    //         .output
    //         .iter()
    //         .filter(|item| item.script_pubkey != change_address.script_pubkey())
    //         .map(|item| item.value)
    //         .sum::<Amount>();

    //     let required_amount = fee + total_output + change_amount;

    //     let res = match total_input.cmp(&required_amount) {
    //         std::cmp::Ordering::Greater => {
    //             CheckTransactionRes::Greater(total_input - required_amount)
    //         }
    //         std::cmp::Ordering::Less => CheckTransactionRes::Less(required_amount - total_input),
    //         std::cmp::Ordering::Equal => CheckTransactionRes::Equal,
    //     };
    //     println!("验证的交易{:#?}", self.transaction);
    //     println!("total input = {}", total_input);
    //     println!("fee  = {}", fee);
    //     println!("out   = {}", total_output);
    //     println!("找零的钱{:#?}", change_amount);
    //     println!("加手续费总结果  = {}", required_amount);
    //     println!("匹配结果{:?}", res);

    //     res
    // }

    // 设置找零以及手续费
    // pub fn set_change_and_fee(
    //     &mut self,
    //     fee_rate: Amount,
    //     address_type: BtcAddressType,
    //     change_address: bitcoin::Address,
    // ) -> crate::Result<usize> {
    //     let mut change_amount1 = Amount::default();
    //     for _i in 0..3 {
    //         // 预估交易的大小
    //         let size = signature::predict_transaction_size_v1(
    //             self.transaction.clone(),
    //             address_type,
    //             &self.multisig_sign_params,
    //         )?;
    //         println!();

    //         println!("size = {}", size);
    //         let fee = fee_rate * size as u64;

    //         match self.check_transaction(fee, change_address.clone(), change_amount1) {
    //             // 大于进行找零
    //             CheckTransactionRes::Greater(change_amount) => {
    //                 println!("进行找零 = {}", change_amount);

    //                 change_amount1 = change_amount;
    //                 self.change_amount(change_address.clone(), change_amount)?
    //             }
    //             // 小于加入新的utxo
    //             CheckTransactionRes::Less(additional_required) => {
    //                 println!("加入新的 = {}", additional_required);
    //                 self.add_utxo(additional_required, change_address.clone())?
    //             }
    //             // 相同构建交易完成
    //             CheckTransactionRes::Equal => return Ok(size),
    //         };
    //     }
    //     Ok(12)
    // }

    // fn change_amount(
    //     &mut self,
    //     change_address: bitcoin::Address,
    //     change_amount: Amount,
    // ) -> crate::Result<()> {
    //     // 检查是否已经存在找零地址并更新金额
    //     if let Some(item) = self
    //         .transaction
    //         .output
    //         .iter_mut()
    //         .find(|item| item.script_pubkey == change_address.script_pubkey())
    //     {
    //         item.value = change_amount;
    //     } else {
    //         self.transaction.output.push(bitcoin::TxOut {
    //             value: change_amount,
    //             script_pubkey: change_address.script_pubkey(),
    //         });
    //     }

    //     println!("找零后的交易 {:#?}", self.transaction);
    //     Ok(())
    // }

    // additional_required :需要的input  = total_input - outinput - free
    // fn add_utxo(
    //     &mut self,
    //     additional_required: Amount,
    //     change_address: bitcoin::Address,
    // ) -> crate::Result<()> {
    //     // The total additional input
    //     let mut additional_input = bitcoin::Amount::from_sat(0);

    //     // UTXOs that have not been selected
    //     let available = self.utxo.available_utxo();
    //     for utxo in available {
    //         additional_input += bitcoin::Amount::from_sat(utxo.value);

    //         self.transaction.input.push(TxIn::from(utxo.clone()));

    //         // Mark this UTXO as used
    //         self.utxo.tag_select(&utxo.txid, utxo.vout);

    //         // If the additional input is sufficient
    //         if additional_input >= additional_required {
    //             break;
    //         }
    //     }

    //     // If all UTXOs have been iterated and there is still not enough money
    //     if additional_input < additional_required {
    //         return Err(crate::UtxoError::InsufficientFee(additional_required.to_btc()).into());
    //     }

    //     // 如果有找零的地址那么进行清空
    //     // self.transaction
    //     //     .output
    //     //     .retain(|item| item.script_pubkey != change_address.script_pubkey());

    //     println!("加入utxo后的交易 {:#?}", self.transaction);

    //     Ok(())
    // }

    pub fn change_and_fee(
        &mut self,
        fee_rate: bitcoin::Amount,
        change_address: bitcoin::Address,
        address_type: BtcAddressType,
        value: bitcoin::Amount,
    ) -> crate::Result<usize> {
        loop {
            // 在预估交易大小是使用交易的副本
            let size = signature::predict_transaction_size(
                self.transaction.clone(),
                Some(change_address.clone()),
                address_type,
                &self.multisig_sign_params,
            )?;

            let res = self.set_transaction_fee(fee_rate, size, value)?;

            // 没有额外的输入进行找零
            if !res.0 {
                self.change(res.1, change_address);
                return Ok(size);
            }
        }
    }

    pub fn spent_all_set_fee(
        &mut self,
        fee_rate: bitcoin::Amount,
        spend_address: bitcoin::Address,
        address_type: BtcAddressType,
    ) -> crate::Result<usize> {
        // 模拟交易的大小
        let size = signature::predict_transaction_size(
            self.transaction.clone(),
            Some(spend_address.clone()),
            address_type,
            &self.multisig_sign_params,
        )?;

        let total_input = self.utxo.total_input_amount();
        let transaction_fee = fee_rate * size as u64;
        if total_input < transaction_fee {
            return Err(crate::UtxoError::InsufficientFee(transaction_fee.to_btc()).into());
        }

        // add spend
        let spend = bitcoin::TxOut {
            value: total_input - transaction_fee,
            script_pubkey: spend_address.script_pubkey(),
        };
        self.transaction.output.push(spend);

        Ok(size)
    }

    fn set_transaction_fee(
        &mut self,
        fee_rate: bitcoin::Amount,
        size: usize,
        value: bitcoin::Amount,
    ) -> crate::Result<(bool, bitcoin::Amount)> {
        // The total amount of selected UTXOs
        let total_input = self.utxo.total_input_amount();

        // transaction fee
        let transaction_fee = fee_rate * size as u64;

        // Whether there is a new input; if there is, the size of the transaction changes,
        // and the fee needs to be recalculated
        let mut has_new_input = false;
        let required_amount = value + transaction_fee;

        // In the case where the current input amount is insufficient
        if total_input < required_amount {
            // How much additional is required
            let additional_required = required_amount - total_input;

            // The total additional input
            let mut additional_input = bitcoin::Amount::from_sat(0);

            // UTXOs that have not been selected
            let available = self.utxo.available_utxo();

            for utxo in available {
                additional_input += bitcoin::Amount::from_sat(utxo.value);

                self.transaction.input.push(TxIn::from(utxo.clone()));
                has_new_input = true;

                // Mark this UTXO as used
                self.utxo.tag_select(&utxo.txid, utxo.vout);

                // If the additional input is sufficient
                if additional_input >= additional_required {
                    break;
                }
            }

            // If all UTXOs have been iterated and there is still not enough money
            if additional_input < additional_required {
                return Err(crate::UtxoError::InsufficientFee(additional_required.to_btc()).into());
            }
        }
        Ok((has_new_input, required_amount))
    }

    // change
    fn change(&mut self, required_amount: Amount, change_address: bitcoin::Address) {
        let total_input = self.utxo.total_input_amount();
        let change = total_input - required_amount;
        if change > Amount::default() {
            self.transaction.output.push(bitcoin::TxOut {
                value: change,
                script_pubkey: change_address.script_pubkey(),
            });
        }
    }

    // 通过input 和out 计算的手续费
    pub fn act_transfer_fee(&self) -> Amount {
        let input_total = self.utxo.total_input_amount();

        let out_total = self
            .transaction
            .output
            .iter()
            .map(|item| item.value)
            .sum::<Amount>();

        input_total - out_total
    }

    // 如果实际的手续费大于 给定的手续费 5倍，那么报错
    pub fn exceeds_max_fee(&self, fee: Amount) -> bool {
        let act_fee = self.act_transfer_fee();

        act_fee > (fee * 5)
    }

    // 转账金额小于手续费
    pub fn is_dust_tx(&self, amount: Amount, fee: Amount) -> bool {
        amount <= (fee / EXPEND_FEE_RATE)
    }

    pub fn get_raw_transaction(&self) -> String {
        consensus::encode::serialize_hex(&self.transaction)
    }
}

#[cfg(test)]
mod tests {
    use super::TransferArg;
    use crate::btc::utxos::{Utxo, UtxoList};

    #[test]
    pub fn condition_1() {
        // 选择了两个utxo 并且所选择的utxo满足了手续费的要求
        let from = "n2xfjp4NfSMWao3V119b5JEU3CKZ7jDZAK";
        let to = "bcrt1qjx3d2sfu5v0jykpzs3a668nf26cgh9awsh7ek9";
        let value = "0.0051";
        let network = wallet_types::chain::network::NetworkKind::Regtest;
        let params = TransferArg::new(from, to, value, Some("p2pkh".to_string()), network).unwrap();

        let mut transaction_build = params.build_transaction(utxos()).unwrap();

        println!(
            "select utxo {:?}",
            transaction_build.utxo.used_utxo_to_hash_map()
        );
        let fee_rate = bitcoin::Amount::from_sat(20);
        let _c = transaction_build
            .change_and_fee(fee_rate, params.from, params.address_type, params.value)
            .unwrap();
        println!("transaction = {:?}", transaction_build.transaction);
    }

    #[test]
    pub fn condition_2() {
        // 选择了两个utxo,选择的手续费utxo不满足手续费的要求，需要在额外的添加一个utxo进来
        let from = "n2xfjp4NfSMWao3V119b5JEU3CKZ7jDZAK";
        let to = "bcrt1qjx3d2sfu5v0jykpzs3a668nf26cgh9awsh7ek9";
        let value = "0.0051";
        let network = wallet_types::chain::network::NetworkKind::Regtest;
        let params = TransferArg::new(from, to, value, Some("p2pkh".to_string()), network).unwrap();

        let mut transaction_build = params.build_transaction(utxos()).unwrap();

        let fee_rate = bitcoin::Amount::from_sat(2700);
        let c = transaction_build
            .change_and_fee(fee_rate, params.from, params.address_type, params.value)
            .unwrap();

        println!(
            "select utxo {:?}",
            transaction_build.utxo.used_utxo_to_hash_map()
        );
        println!("transaction = {:?}", transaction_build.transaction);
        println!("size  = {}", c);
        println!(
            "total input  = {}",
            transaction_build.utxo.total_input_amount()
        );
        println!("transaction fee   = {}", fee_rate * c as u64);
    }

    pub fn utxos() -> UtxoList {
        let utxo_list = UtxoList(vec![
            // 0.005
            Utxo {
                txid: "ed1172b141a9aac076dbc36ba1cf791a48edde46028ee5d68527d822789691ca"
                    .to_string(),
                vout: 1,
                value: 500000,
                confirmations: 10,
                selected: false,
            },
            // 0.01
            Utxo {
                txid: "53a87b9b72759775f874ae99c1d786dc22623c1a23661052e848d12de75e875f"
                    .to_string(),
                vout: 2,
                value: 1000000,
                confirmations: 20,
                selected: false,
            },
            // 0.002
            Utxo {
                txid: "f46a144b21aa41ba1d997784c719ab56c51c5a353b85a732ed54968b4d41c81d"
                    .to_string(),
                vout: 3,
                value: 200000,
                confirmations: 15,
                selected: false,
            },
            // 0.015
            Utxo {
                txid: "2534a19eb5765a53269c7a1c3cd457496cb2b4c1bdbade8dc6265354a7e49818"
                    .to_string(),
                vout: 4,
                value: 1500000,
                confirmations: 30,
                selected: false,
            },
            // 0.0025
            Utxo {
                txid: "9b1d532555b23b5b55496492c91ee2c3691db8e9e9e512f65a2452cb1b78172b"
                    .to_string(),
                vout: 5,
                value: 250000,
                confirmations: 5,
                selected: false,
            },
        ]);
        utxo_list
    }

    // #[test]
    // pub fn test_v1_condition_1() {
    //     // 选择了两个utxo 并且所选择的utxo满足了手续费的要求
    //     let from = "n2xfjp4NfSMWao3V119b5JEU3CKZ7jDZAK";
    //     let to = "bcrt1qjx3d2sfu5v0jykpzs3a668nf26cgh9awsh7ek9";
    //     let value = "0.0051";

    //     let network = wallet_types::chain::network::NetworkKind::Regtest;
    //     let params = TransferArg::new(from, to, value, Some("p2pkh".to_string()), network).unwrap();

    //     let mut build = params.build_transaction_v1(utxos()).unwrap();

    //     let fee_rate = bitcoin::Amount::from_sat(20);
    //     let size = build
    //         .set_change_and_fee(fee_rate, params.address_type, params.change_address)
    //         .unwrap();
    //     println!("size = {}", size);
    // }
}
