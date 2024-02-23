use unicode_data::{codepoint::*, UNICODE};

use crate::output::stats::CodepointGroups;

/// стартер без декомпозиции
pub const MARKER_STARTER: u64 = 0b_000;
/// нестартер без декомпозиции
pub const MARKER_NONSTARTER: u64 = 0b_001;
/// 16-битная пара
pub const MARKER_PAIR: u64 = 0b_010;
/// синглтон
pub const MARKER_SINGLETON: u64 = 0b_011;
/// декомпозиция, вынесенная во внешний блок
pub const MARKER_EXPANSION: u64 = 0b_100;
/// слог хангыль
pub const MARKER_HANGUL: u64 = 0b_101;

/// закодировать кодпоинт для таблицы данных
pub fn encode_codepoint(
    codepoint: &Codepoint,
    canonical: bool,
    expansion_position: usize,
    stats: &mut CodepointGroups,
) -> (u64, Vec<u32>)
{
    let decomposition = match canonical {
        true => &codepoint.canonical_decomposition,
        false => &codepoint.compat_decomposition,
    };

    let variants = &[
        starter,
        nonstarter,
        singleton,
        pair,
        triple16,
        // pair18,
        starter_to_nonstarters,
        nonstarter_decomposition,
        triple18,
        long_decomposition,
    ];

    let value = variants
        .iter()
        .find_map(|f| f(codepoint, decomposition, expansion_position, stats));

    match value {
        Some(value) => value,
        None => {
            // не подошёл ни один из вариантов

            let dec_string: String = decomposition
                .iter()
                .map(|c| format!("U+{:04X} [{}] ", *c, get_ccc(*c).compressed()))
                .collect();

            panic!(
                "\n\nне определили тип хранения кодпоинта: U+{:04X} - {} [CCC={}] -> {}\n\n",
                codepoint.code,
                codepoint.name,
                u8::from(codepoint.ccc),
                dec_string,
            );
        }
    }
}

/// стартер:
///     - CCC = 0
///     - нет декомпозиции
fn starter(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    _: usize,
    _: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.is_starter() || !decomposition.is_empty() {
        return None;
    }

    let value = MARKER_STARTER;

    Some((value, vec![]))
}

/// нестартер:
///     - ССС > 0
///     - нет декомпозиции
fn nonstarter(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    _: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.is_nonstarter() || !decomposition.is_empty() {
        return None;
    }

    let code = codepoint.code as u64;
    let ccc = codepoint.ccc.compressed() as u64;

    let value = MARKER_NONSTARTER | (ccc << 8) | (code << 16);

    to_stats(stats, "1. нестартеры", codepoint, decomposition);
    Some((value, vec![]))
}

/// синглтон:
///     - стартер
///     - декомпозиция из одного стартера
fn singleton(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    _: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.is_starter()
        || decomposition.len() != 1
        || !get_ccc(decomposition[0]).is_starter()
    {
        return None;
    }

    let code = decomposition[0] as u64;

    let value = MARKER_SINGLETON | (code << 16);

    to_stats(stats, "2. синглтоны", codepoint, decomposition);
    Some((value, vec![]))
}

/// пара:
///     - стартер
///     - декомпозиция из 2х кодпоинтов
///     - первый из них - стартер
fn pair(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    _: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.is_starter()
        || decomposition.len() != 2
        // || decomposition.iter().any(|&c| c > 0xFFFF)
        || !get_ccc(decomposition[0]).is_starter()
    {
        return None;
    }

    let c1 = decomposition[0] as u64;
    let c2 = decomposition[1] as u64;
    let c2_ccc = get_ccc(decomposition[1]).compressed() as u64;

    let value = MARKER_PAIR | (c1 << 8) | (c2_ccc << 32) | (c2 << 40);

    to_stats(stats, "3. пары", codepoint, decomposition);
    Some((value, vec![]))
}

/// тройка (16 бит):
///     - стартер
///     - декомпозиция из 3х кодпоинтов
///     - кодпоинты декомпозиции - 16-битные
///     - первый из них - стартер
fn triple16(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    _: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.is_starter()
        || decomposition.len() != 3
        || decomposition.iter().any(|&c| c > 0xFFFF)
        || !get_ccc(decomposition[0]).is_starter()
    {
        return None;
    }

    let c1 = decomposition[0] as u64;
    let c2 = decomposition[1] as u64;
    let c3 = decomposition[2] as u64;

    let c2_ccc = get_ccc(decomposition[1]).compressed() as u64;
    let c3_ccc = get_ccc(decomposition[2]).compressed() as u64;

    let value = c1 | (c2_ccc << 16) | (c2 << 24) | (c3_ccc << 40) | (c3 << 48);

    to_stats(stats, "4. тройки (16 бит)", codepoint, decomposition);
    Some((value, vec![]))
}

// /// пара (18 бит):
// ///     - стартер
// ///     - декомпозиция из 2х кодпоинтов
// ///     - хотя бы один из кодпоинтов декомпозиции - 18-битный
// ///     - первый из них - стартер
// fn pair18(
//     codepoint: &Codepoint,
//     decomposition: &Vec<u32>,
//     expansion_position: usize,
//     stats: &mut CodepointGroups,
// ) -> Option<(u64, Vec<u32>)>
// {
//     if !codepoint.is_starter()
//         || decomposition.len() != 2
//         || decomposition.iter().all(|&c| c <= 0xFFFF)
//         || !get_ccc(decomposition[0]).is_starter()
//     {
//         return None;
//     }

//     let value = MARKER_EXPANSION
//         | ((decomposition.len() as u64) << 8)
//         | ((expansion_position as u64) << 16);

//     to_stats(stats, "5. пары (18 бит)", codepoint, decomposition);
//     Some((value, map_expansion(decomposition)))
// }

/// стартер с декомпозицией в нестартеры
///     - стартер
///     - есть декомпозиция, которая состоит из нестартеров
fn starter_to_nonstarters(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    expansion_position: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.is_starter()
        || decomposition.is_empty()
        || !decomposition.iter().all(|&c| get_ccc(c).is_nonstarter())
    {
        return None;
    }

    let value = MARKER_EXPANSION
        | ((decomposition.len() as u64) << 8)
        | ((expansion_position as u64) << 16);

    to_stats(stats, "6. стартеры в нестартеры", codepoint, decomposition);
    Some((value, map_expansion(decomposition)))
}

/// нестартер с декомпозицией
///     - нестартер
///     - есть декомпозиция
fn nonstarter_decomposition(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    expansion_position: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.is_nonstarter() || decomposition.is_empty() {
        return None;
    }

    let value = MARKER_EXPANSION
        | ((decomposition.len() as u64) << 8)
        | ((expansion_position as u64) << 16);

    to_stats(
        stats,
        "7. нестартеры с декомпозицией",
        codepoint,
        decomposition,
    );
    Some((value, map_expansion(decomposition)))
}

/// тройка (18 бит)
///     - стартер
///     - декомпозиция в 3 кодпоинта
///     - хотя-бы один из них - 18 бит
///     - декомпозиция начинается со стартера
fn triple18(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    expansion_position: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.is_starter()
        || decomposition.len() != 3
        || decomposition.iter().all(|&c| c <= 0xFFFF)
        || !get_ccc(decomposition[0]).is_starter()
    {
        return None;
    }

    let value = MARKER_EXPANSION
        | ((decomposition.len() as u64) << 8)
        | ((expansion_position as u64) << 16);

    to_stats(stats, "8. тройки (18 бит)", codepoint, decomposition);
    Some((value, map_expansion(decomposition)))
}

/// декомпозиция > 3 символов
///     - стартер
///     - декомпозиция > 3 кодпоинтов
///     - декомпозиция начинается со стартера
fn long_decomposition(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    expansion_position: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.is_starter()
        || decomposition.len() <= 3
        || !get_ccc(decomposition[0]).is_starter()
    {
        return None;
    }

    let value = MARKER_EXPANSION
        | ((decomposition.len() as u64) << 8)
        | ((expansion_position as u64) << 16);

    to_stats(stats, "9. длинная декомпозиция", codepoint, decomposition);
    Some((value, map_expansion(decomposition)))
}

// ----

/// получаем CCC кодпоинта
fn get_ccc(codepoint: u32) -> CanonicalCombiningClass
{
    match UNICODE.get(&codepoint) {
        Some(codepoint) => codepoint.ccc,
        None => CanonicalCombiningClass::NotReordered,
    }
}

/// преобразовать декомпозицию в вектор значений, состоящих из кодпоинта (младшие биты) и CCC (8 старших бит)
fn map_expansion(decomposition: &[u32]) -> Vec<u32>
{
    decomposition
        .iter()
        .map(|e| (*e << 8) | get_ccc(*e).compressed() as u32)
        .collect()
}

/// строка с данными о кодпоинте для статистики
fn info(codepoint: &Codepoint, decomposition: &[u32]) -> String
{
    let dec_string: String = decomposition
        .iter()
        .map(|c| format!("[{}] ", u8::from(get_ccc(*c))))
        .collect();

    format!(
        "U+{:04X} - {} [{}] ({}) {}\n",
        codepoint.code,
        codepoint.name,
        u8::from(codepoint.ccc),
        decomposition.len(),
        dec_string,
    )
}

/// пишем в статистику
fn to_stats<'a>(
    stats: &mut CodepointGroups<'a>,
    group: &'a str,
    codepoint: &Codepoint,
    decomposition: &[u32],
)
{
    stats
        .entry(group)
        .or_insert(vec![])
        .push(info(codepoint, decomposition));
}
