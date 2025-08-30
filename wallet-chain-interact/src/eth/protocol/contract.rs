use alloy::sol;

sol!(
    function balanceOf(address owner) public view returns (uint256 balance);
    function transfer(address from,uint256 amount) public view returns (bool res);
    function mint(address to, uint256 amounts) external;
    function decimals() pub view return (uint8);
    function symbol() public view returns (string);
    function name() public view returns (string);
    function isBlackListed(address from) public view returns (bool);
    function deposit() public payable;
    function approve(address spender, uint256 amount) public returns (bool);
    function allowance(address owner, address spender) public view returns (uint256);

    function withdraw(uint256 amount) public;

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
