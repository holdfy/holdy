package services

// PostgreSQL INTEGER (int32): colunas amount_cents nas tabelas do banco.
const (
	maxAmountCentsInt4 = int64(2147483647)
	minAmountCentsDB   = int64(1)
)

func clampAmountCentsForDB(cents int64) int64 {
	if cents > maxAmountCentsInt4 {
		return maxAmountCentsInt4
	}
	if cents < minAmountCentsDB {
		return minAmountCentsDB
	}
	return cents
}
