use alloy::sol;

sol!(
    function balanceOf(address owner) public view returns (uint256 balance);
    function transfer(address from,uint256 amount) public view returns (bool res);
    function mint(address to, uint256 amounts) external;
    function decimals() pub view return (uint8);
    function symbol() public view returns (string);
    function name() public view returns (string);
    function isBlackListed(address from) public view returns (bool);

    function createProxyWithNonce(address _singleton, bytes memory initializer, uint256 saltNonce) public returns (address proxy);

    function setup(
        address[] calldata _owners,
        uint256 _threshold,
        address to,
        bytes calldata data,
        address fallbackHandler,
        address paymentToken,
        uint256 payment,
        address payable paymentReceiver
    ) external override;


    function getTransactionHash(
        address to,
        uint256 value,
        bytes calldata data,
        uint8 operation,
        uint256 safeTxGas,
        uint256 baseGas,
        uint256 gasPrice,
        address gasToken,
        address refundReceiver,
        uint256 _nonce
    ) public view override returns (bytes32);

    function execTransaction(
        address to,
        uint256 value,
        bytes calldata data,
        uint8 operation,
        uint256 safeTxGas,
        uint256 baseGas,
        uint256 gasPrice,
        address gasToken,
        address payable refundReceiver,
        bytes memory signatures
    ) external payable override returns (bool success);

    function nonce() public view returns (uint256 nonce);

    function proxyCreationCode() public pure returns (bytes memory);
);

// // eth多签的内部交易数据结构
// #[derive(Debug, Serialize, Deserialize)]
// pub struct MultisigTxInternal {
//     pub internal: String,
//     pub hash_message: String,
// }

// // TODO 封装util bincode ;
// impl MultisigTxInternal {
//     pub fn new(internal: String, hash_message: String) -> Self {
//         Self {
//             internal,
//             hash_message,
//         }
//     }
//     pub fn to_string(&self) -> crate::Result<String> {
//         let bytes = bincode::serialize(&self).unwrap();
//         Ok(hex::encode(bytes))
//     }

//     pub fn from_str(data: &str) -> crate::Result<Self> {
//         let bytes = wallet_utils::hex_func::hex_decode(data)?;
//         let res = bincode::deserialize::<MultisigTxInternal>(&bytes).unwrap();
//         Ok(res)
//     }
// }
