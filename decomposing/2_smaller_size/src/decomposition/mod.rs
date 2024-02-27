pub mod hangul;

/// последний кодпоинт с декомпозицией
pub const LAST_DECOMPOSING_CODEPOINT: u32 = 0x2FA1D;

/// нестартер без декомпозиции
pub const MARKER_NONSTARTER: u8 = 1;
/// синглтон
pub const MARKER_SINGLETON: u8 = 2;
/// декомпозиция, вынесенная во внешний блок
pub const MARKER_EXPANSION: u8 = 3;
/// слог хангыль
pub const MARKER_HANGUL: u8 = 4;

pub enum DecompositionValue
{
    /// декомпозиция на 2 кодпоинта, первый - стартер
    Pair((u16, u16)),
    /// нестартер (например, диакретический знак)
    Nonstarter(u8),
    /// синглтон (стартер, декомпозирующийся в другой стартер)
    Singleton(u32),
    /// декомпозиция на несколько символов, в параметрах - индекс первого элемента в дополнительной таблице и количество этих элементов
    Expansion(u16, u8),
    /// слог хангыль
    Hangul,
}

/// парсим значение из таблицы
#[inline(always)]
pub fn parse_data_value(value: u32) -> DecompositionValue
{
    match value as u8 {
        MARKER_NONSTARTER => parse_nonstarter(value),
        MARKER_SINGLETON => parse_singleton(value),
        MARKER_EXPANSION => parse_expansion(value),
        MARKER_HANGUL => DecompositionValue::Hangul,
        _ => parse_pair(value),
    }
}

/// нестартер без декомпозиции
#[inline(always)]
fn parse_nonstarter(value: u32) -> DecompositionValue
{
    DecompositionValue::Nonstarter((value >> 8) as u8)
}

/// синглтон
#[inline(always)]
fn parse_singleton(value: u32) -> DecompositionValue
{
    DecompositionValue::Singleton(value >> 8)
}

/// пара
#[inline(always)]
fn parse_pair(value: u32) -> DecompositionValue
{
    DecompositionValue::Pair((value as u16, (value >> 16) as u16))
}

/// декомпозиция, вынесенная во внешний блок
#[inline(always)]
fn parse_expansion(value: u32) -> DecompositionValue
{
    DecompositionValue::Expansion((value >> 16) as u16, (value >> 8) as u8)
}
