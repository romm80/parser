# Transaction Parser

Библиотека и CLI-утилиты для парсинга, сериализации и конвертации финансовых транзакций между различными форматами данных.

## Возможности

- 📊 Поддержка трех форматов: CSV, Text (key-value), Binary
- 🔄 Конвертация между любыми форматами
- 📝 Работа с файлами, stdin/stdout и любыми источниками данных

## Быстрый старт

### Использование библиотеки

```rust
use parser::{read, write, Format};
use std::fs::File;

// Чтение транзакций из CSV
let file = File::open("transactions.csv")?;
let transactions = read(file, Format::Csv)?;

// Запись в текстовый формат
let output = File::create("output.txt")?;
write(output, Format::Txt, transactions)?;
```

### CLI-утилиты

**Конвертация форматов:**

```bash
# CSV → Text
cargo run --bin convrter-cli -- \
  --input data.csv --input-format csv \
  --output data.txt --output-format txt

# Использование stdin/stdout
cat data.bin | cargo run --bin convrter-cli -- \
  --input-format bin --output-format csv > data.csv
```

**Сравнение файлов:**

```bash
# Сравнить транзакции в разных форматах
cargo run --bin comparer-cli -- \
  --file1 data.bin --file1-format bin \
  --file2 data.csv --file2-format csv
```

## Форматы данных

### CSV

CSV файл в кодировке UTF-8 с обязательным заголовком:

```csv
TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
```

- Первая строка — строгий заголовок с именами полей
- Поля разделены запятыми
- `DESCRIPTION` всегда в двойных кавычках
- Пустые строки игнорируются

### Text

Текстовый формат в виде пар ключ-значение:

```
# Record 1 (DEPOSIT)
TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"
```

- Формат: `KEY: VALUE`
- Поля могут быть в любом порядке
- Записи разделяются пустыми строками
- Комментарии (начинаются с `#`) игнорируются
- `DESCRIPTION` в двойных кавычках

### Binary

Компактный бинарный формат (big-endian):

**Структура записи:**
```
[Заголовок записи]
  4 байта: MAGIC (0x59 0x50 0x42 0x4E = 'YPBN')
  4 байта: RECORD_SIZE (размер тела записи)

[Тело записи]
  8 байт:  TX_ID (u64)
  1 байт:  TX_TYPE (0=DEPOSIT, 1=TRANSFER, 2=WITHDRAWAL)
  8 байт:  FROM_USER_ID (u64, 0 для DEPOSIT)
  8 байт:  TO_USER_ID (u64, 0 для WITHDRAWAL)
  8 байт:  AMOUNT (u64)
  8 байт:  TIMESTAMP (u64, миллисекунды)
  1 байт:  STATUS (0=SUCCESS, 1=FAILURE, 2=PENDING)
  4 байта: DESC_LEN (u32)
  N байт:  DESCRIPTION (UTF-8)
```

Файл — последовательность таких записей. MAGIC-значение позволяет синхронизироваться при повреждении данных.

## API библиотеки

### Основные функции

```rust
/// Чтение транзакций из любого источника
pub fn read<R: Read>(data: R, format: Format)
    -> Result<Vec<Transaction>, ParseError>

/// Запись транзакций в любой приемник
pub fn write<W: Write>(data: W, format: Format, tx: Vec<Transaction>)
    -> Result<(), WriteError>
```

### Типы данных

```rust
pub struct Transaction {
    pub tx_id: u64,
    pub tx_type: TransactionType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: u64,
    pub timestamp: u64,
    pub status: Status,
    pub description: String,
}

pub enum TransactionType {
    DEPOSIT,
    TRANSFER,
    WITHDRAWAL,
}

pub enum Status {
    SUCCESS,
    FAILURE,
    PENDING,
}

pub enum Format {
    Csv,
    Txt,
    Bin,
}
```

### Обработка ошибок

```rust
use parser::{read, Format, ParseError};

match read(file, Format::Csv) {
    Ok(transactions) => {
        println!("Загружено {} транзакций", transactions.len());
    }
    Err(ParseError::InvalidHeaders(msg)) => {
        eprintln!("Неверные заголовки CSV: {}", msg);
    }
    Err(ParseError::FieldValueNotFound(field)) => {
        eprintln!("Отсутствует поле: {}", field);
    }
    Err(ParseError::InvalidTextFieldValue { field, value }) => {
        eprintln!("Неверное значение поля {}: {}", field, value);
    }
    Err(e) => eprintln!("Ошибка парсинга: {}", e),
}
```

## Примеры использования

### Конвертация форматов

```rust
use parser::{read, write, Format};
use std::fs::File;

fn convert_csv_to_binary(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input = File::open(input_path)?;
    let transactions = read(input, Format::Csv)?;

    let output = File::create(output_path)?;
    write(output, Format::Bin, transactions)?;

    Ok(())
}
```

### Фильтрация транзакций

```rust
use parser::{read, write, Format, Status};
use std::fs::File;

fn filter_successful(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input = File::open(input_path)?;
    let transactions = read(input, Format::Csv)?;

    let successful: Vec<_> = transactions
        .into_iter()
        .filter(|tx| tx.status == Status::SUCCESS)
        .collect();

    let output = File::create(output_path)?;
    write(output, Format::Csv, successful)?;

    Ok(())
}
```

## CLI Reference

### converter-cli

Конвертирует транзакции между форматами.

**Опции:**
- `--input <PATH>` - входной файл (по умолчанию stdin)
- `--input-format <FORMAT>` - формат входа: `csv`, `txt`, `bin`
- `--output <PATH>` - выходной файл (по умолчанию stdout)
- `--output-format <FORMAT>` - формат выхода: `csv`, `txt`, `bin`

**Примеры:**

```bash
# Файл → Файл
cargo run --bin convrter-cli -- \
  --input data.csv --input-format csv \
  --output data.bin --output-format bin

# stdin → Файл
cat data.txt | cargo run --bin convrter-cli -- \
  --input-format txt \
  --output data.csv --output-format csv

# Файл → stdout
cargo run --bin convrter-cli -- \
  --input data.bin --input-format bin \
  --output-format txt
```

### comparer-cli

Сравнивает транзакции из двух файлов.

**Опции:**
- `--file1 <PATH>` - путь к первому файлу
- `--file1-format <FORMAT>` - формат первого файла
- `--file2 <PATH>` - путь ко второму файлу
- `--file2-format <FORMAT>` - формат второго файла

**Примеры:**

```bash
# Сравнение разных форматов
cargo run --bin comparer-cli -- \
  --file1 original.bin --file1-format bin \
  --file2 converted.csv --file2-format csv

# Вывод при совпадении:
# transaction are identical

# Вывод при отсутствии транзакции во втором файле:
# transaction id:1000000000000000 in file 'original.bin' not found in file 'converted.csv'

# Вывод при отсутствии транзакции в первом файле:
# transaction id:1000000000000001 in file 'original.bin' not found in file 'converted.csv'

# Вывод при различиях в данных:
# transaction id:1000000000000002 in file 'original.bin' not equal transaction id:1000000000000002 in file 'converted.csv'
```
