# nbxdca

A straightforward CLI tool for automated Dollar Cost Averaging (DCA) on [nbx.com](https://nbx.com)

## Installation

```
git clone https://github.com/root4loot/nbxdca.git
cd nbxdca
```

```
cargo build --release
```

```
sudo mv target/release/nbxdca /usr/local/bin/
```

## Configuring API Credentials

1. **Create API Key:** Head to [NBC Account Settings](https://app.nbx.com/account/api) and create a new key.
2. **Configure config.toml**: Create a config.toml file and enter the credentials as follows:

```
id = "your_id"
key = "your_key"
passphrase = "your_passphrase"
secret = "your_secret"
token_lifetime = "1"  # Token lifetime in hours
```

Replace your_id, your_key, your_passphrase, and your_secret with the values obtained from NBX. Ensure to refer to this file (using the `-c | --config` flag) when running the program.

Example:

```sh
nbxdca -c /path/to/config.toml account balance BTC
```



## Usage

```sh
Usage: nbxdca [OPTIONS] [COMMAND]

Commands:
  account  Account command
  order    Order command
  help     Print this message or the help of the given subcommand(s)

Options:
  -c, --config <FILE>  Path to config.toml (Default ./config.toml)
  -h, --help           Print help information
  -V, --version        Print version information
```

### Account

```sh
Usage: nbxdca account [COMMAND]

Commands:
  balance  get account balance
  help     Print this message or the help of the given subcommand(s)
```

```sh
Usage: nbxdca account balance <ASSET>

Arguments:
  <ASSET>  Asset ID

```

### Order

```sh
Usage: nbxdca order [COMMAND]

Commands:
  new      place new order
  details  print details for last order
  help     Print this message or the help of the given subcommand(s)
```

```sh
Usage: nbxdca order new <SIDE> <TICKER> <AMOUNT>

Arguments:
  <SIDE>    Order side (BUY or SELL)
  <TICKER>  Ticker symbol (BTC, ETH, etc)
  <AMOUNT>  Amount in fiat currency for BUY orders or quantity in crypto for SELL orders
```

## Automated DCA Setup Using Crontab

Automate your DCA strategy for `nbxdca` with crontab by scheduling daily DCA in (buy) and DCA out (sell) operations.

- **Project Binary in PATH**: Ensure that the `nbxdca` binary is added to your system's PATH.
- **Cron Service**: Make sure that the cron service is active and running on your system.
- **Timezone Settings**: Confirm that your system's timezone is correctly set, as cron jobs depend on the system's local time.

### DCA In: Buy BTC Daily
To purchase BTC worth 300 NOK every day at 8:00 AM:

```
0 8 * * * /path/to/nbxdca -c /path/to/config.toml order new BUY BTC 300.00 >> /var/log/nbxdca.log 2>&1
```

### DCA Out: Sell BTC Daily

To sell 0.0000023 BTC every day at 8:00 AM:

```
0 8 * * * /path/to/nbxdca -c /path/to/config.toml order new SELL BTC 0.0000023 >> /var/log/nbxdca.log 2>&1
```

## Examples

**Print account balance (NOK)**

```sh
nbxdca -c config.toml account balance NOK

10000
```

**Place Buy order for Bitcoin (BTC):**  
Execute a market buy order for Bitcoin (BTC) with a total investment of 300 NOK at the current market price.

```sh
nbxdca -c config.toml order new BUY BTC 300.00

Create new order: Side: BUY, Ticker: BTC, Amount: 300.00
Order created successfully
```

**Place Sell Order for Bitcoin (BTC):**  
Execute a market sell order for a specific quantity of Bitcoin (BTC), in this case, 0.0000023 BTC at the current market price to NOK.

```sh
nbxdca -c config.toml order new SELL BTC 0.0000023

Create new order: Side: SELL, Ticker: BTC, Amount: 0.0000023
Order created successfully
```

**Get details for last order**

```sh
nbxdca -c config.toml order details 

Order { created: "2024-01-01T07:00:02.019000+00:00", fee: 2.08, price: 435387.4, quantity: 0.00068425, cost: 297.91382 }
```
