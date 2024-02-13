use decomposition::hangul::*;
use decomposition::*;

mod data;
mod decomposition;
mod macros;

/// нормализатор NF(K)D
#[repr(align(32))]
pub struct DecomposingNormalizer<'a> {
    /// индекс блока. u8 достаточно, т.к. в NFD последний блок - 0x7E, в NFKD - 0xA6
    index: &'a [u8],
    /// основные данные
    data: &'a [u64],
    /// данные кодпоинтов, которые не вписываются в основную часть
    expansions: &'a [u32],
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    continuous_block_end: u32,
}

impl<'a> From<data::DecompositionData<'a>> for DecomposingNormalizer<'a> {
    fn from(source: data::DecompositionData<'a>) -> Self {
        Self {
            index: source.index,
            data: source.data,
            expansions: source.expansions,
            continuous_block_end: source.continuous_block_end,
        }
    }
}

impl<'a> DecomposingNormalizer<'a> {
    /// NFD-нормализатор
    pub fn nfd() -> Self {
        Self::from(data::nfd())
    }

    /// NFKD-нормализатор
    pub fn nfkd() -> Self {
        Self::from(data::nfkd())
    }

    /// нормализация строки
    /// исходная строка должна являться well-formed UTF-8 строкой
    #[inline(never)]
    pub fn normalize(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut buffer: Vec<Codepoint> = Vec::with_capacity(18);

        for char in input.chars() {
            let code = u32::from(char);

            // у символа может быть декомпозиция: стартеры пишем в результат, нестартеры - в буфер,
            // если после нестартера встречаем стартер - сортируем нестартеры, записываем, сбрасываем буфер

            match self.decompose(code) {
                DecompositionValue::None => {
                    flush_buffer(&mut result, &mut buffer);
                    write!(result, code);
                }
                DecompositionValue::Nonstarter(ccc) => buffer.push(Codepoint { code, ccc }),
                DecompositionValue::Pair(c1, c2) => {
                    flush_buffer(&mut result, &mut buffer);
                    write!(result, c1);

                    match c2.ccc == 0 {
                        true => write!(result, c2.code),
                        false => buffer.push(c2),
                    }
                }
                DecompositionValue::Triple(c1, c2, c3) => {
                    flush_buffer(&mut result, &mut buffer);
                    write!(result, c1);

                    if c3.ccc == 0 {
                        write!(result, c2.code, c3.code);
                    } else {
                        match c2.ccc == 0 {
                            true => write!(result, c2.code),
                            false => buffer.push(c2),
                        }
                        buffer.push(c3);
                    }
                }
                DecompositionValue::Singleton(c1) => {
                    flush_buffer(&mut result, &mut buffer);
                    write!(result, c1);
                }
                DecompositionValue::Expansion(index, count) => {
                    for entry in
                        &self.expansions[(index as usize)..(index as usize + count as usize)]
                    {
                        let ccc = c32_ccc!(entry);
                        let code = c32_code!(entry);

                        match ccc == 0 {
                            true => {
                                flush_buffer(&mut result, &mut buffer);
                                write!(result, code);
                            }
                            false => buffer.push(Codepoint { code, ccc }),
                        }
                    }
                }
                DecompositionValue::Hangul => {
                    flush_buffer(&mut result, &mut buffer);
                    decompose_hangul(&mut result, code);
                }
            }
        }

        flush_buffer(&mut result, &mut buffer);

        result
    }

    /// получить декомпозицию символа
    #[inline(always)]
    fn decompose(&self, code: u32) -> DecompositionValue {
        // все кодпоинты, следующие за U+2FA1D не имеют декомпозиции
        if code > LAST_DECOMPOSING_CODEPOINT {
            return DecompositionValue::None;
        };

        parse_data_value(self.get_decomposition_value(code))
    }

    /// данные о декомпозиции символа
    #[inline(always)]
    fn get_decomposition_value(&self, code: u32) -> u64 {
        match code <= self.continuous_block_end {
            true => self.data[code as usize],
            false => {
                let block_index = (code >> 7) as usize;
                let block = self.index[block_index] as usize;

                let block_offset = block << 7;
                let code_offset = ((code as u8) & 0x7F) as usize;

                let index = block_offset | code_offset;

                self.data[index]
            }
        }
    }
}

/// отсортировать кодпоинты буфера по CCC, записать в результат и освободить буфер
#[inline(always)]
fn flush_buffer(result: &mut String, buffer: &mut Vec<Codepoint>) {
    if !buffer.is_empty() {
        if buffer.len() > 1 {
            buffer.sort_by_key(|c| c.ccc);
        }

        for codepoint in buffer.iter() {
            write!(result, codepoint.code);
        }

        buffer.clear();
    };
}
