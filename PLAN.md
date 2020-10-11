# Plan

### Tasks:

1. Build a vanity name registering system resistant against frontrunning.
2. You can make reasonable assumptions on the size, encoding of the name. (size: 1-63 chars, encoding: UTF-8)
3. An unregistered name can be registered for a certain amount of time by locking a certain balance of an account. (price: floor(PRICE/size), period: payment/price, a period should be between MIN_LOCKING_PERIOD and less MAX_LOCKING_PERIOD)
4. After the registration expires, the account loses ownership of the name and his balance is unlocked. (stores (block, count of names) in storage, removes them on_initialize)
5. The registration can be renewed by making an on-chain call to keep the name registered and balance locked. (can be called anytime, even right after first reservation)
6. You can assume reasonable defaults for the locking amount and period. (PRICE: 1000, MIN_LOCKING_PERIOD and MAX_LOCKING_PERIOD are in blocks, default to 365 and 365*10 correspondingly)
7. Also, a malicious node/validator should not be able to front-run the process by censoring transactions of an honest user and registering its name in its own account.

### Modules:

* pallet-vanity-name - Registering system that will handle registering and auto unregistering logic. Also, will store registared names in storage.
