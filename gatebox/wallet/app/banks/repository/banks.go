package banksRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type BanksRepositoryIF interface {
     GetBanks(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetBanksById(ctx context.Context, id int64) (*model.Banks, error)
     GetBanksByBankCodeInternal(ctx context.Context, bankcodeinternal string) (*model.Banks, error)
     InsertBanks(ctx context.Context, banks *model.Banks) (int64, error)
     UpdateBanks(ctx context.Context, banks *model.Banks, id int64) error
     DeleteBanksById(ctx context.Context, id int64) (bool, error)
}
 type BanksRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewBanksRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *BanksRepository{
    return &BanksRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Banks"),
     }
}
func (t BanksRepository)  GetBanks(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("BanksRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetBanks")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetBanks.offset", offset)
	tracker.AddParam("repository.GetBanks.limit", limit)
	itemsPage 			= model.ItemsPage{}
	bankss := []model.Banks{}

	rows, err := t.PGRead.Query(ctx, SQL_BANKS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "BanksRepository.repository.GetBankss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var banks model.Banks
		err := rows.Scan(
			&banks.ID,
			&banks.BankCodeInternal,
			&banks.BankCode,
			&banks.Name,
			&banks.FullName,
			&banks.Website,
			&banks.IsOpenFinance,
			&banks.IsActive,
			&banks.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "BanksRepository.repository.GetBankss.Scan: ", err.Error())
			return itemsPage, err
		}
		bankss = append(bankss, banks)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "BanksRepository.repository.GetBankss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(bankss) > 0 {
		qtyRecords = bankss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = bankss

	tracker.AddResult("repository.GetBanks.rows_returned", len(bankss))
	tracker.AddResult("repository.GetBanks.total_count", len(bankss))

	return itemsPage, nil
}
func (t BanksRepository)  GetBanksById(ctx context.Context, id int64) (banks *model.Banks, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("BanksRepository -> GetBanksById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetBanksById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetBanksById.id", id)

	banks = new(model.Banks)
	row := t.PGRead.QueryRow(ctx, SQL_GET_BANKS_BY_ID, id)
		err = row.Scan(
			&banks.ID,
			&banks.BankCodeInternal,
			&banks.BankCode,
			&banks.Name,
			&banks.FullName,
			&banks.Website,
			&banks.IsOpenFinance,
			&banks.IsActive,
			&banks.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"BanksRepository.repository.GetBanksById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetBanksById.found", true)
	return banks, nil
}
func (t BanksRepository)  GetBanksByBankCodeInternal(ctx context.Context, bankcodeinternal string) (banks *model.Banks, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("BanksRepository -> GetBanksByBankCodeInternal", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetBanksByBankCodeInternal")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetBanksByBankCodeInternal.bankcodeinternal", bankcodeinternal)

	banks = new(model.Banks)
	row := t.PGRead.QueryRow(ctx, SQL_GET_BANKS_BY_BANK_CODE_INTERNAL, bankcodeinternal)
		err = row.Scan(
			&banks.ID,
			&banks.BankCodeInternal,
			&banks.BankCode,
			&banks.Name,
			&banks.FullName,
			&banks.Website,
			&banks.IsOpenFinance,
			&banks.IsActive,
			&banks.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"BanksRepository.repository.GetBanksBybankcodeinternal: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return banks, nil
}
func (t BanksRepository)  DeleteBanksById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("BanksRepository -> DeleteBanksById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteBanksById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_BANKS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"BanksRepository.repository.DeleteBanksById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteBanksById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteBanksById.deleted", result)
	return true, err
}
func (t BanksRepository)  InsertBanks(ctx context.Context,banks *model.Banks) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("BanksRepository -> InsertBanks", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertBanks")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertBanks.bankcodeinternal", banks.BankCodeInternal)

	err = t.PGWrite.QueryRow(ctx,  SQL_BANKS_INSERT,
			banks.BankCodeInternal,
			banks.BankCode,
			banks.Name,
			banks.FullName,
			banks.Website,
			banks.IsOpenFinance,
			banks.IsActive,
			banks.CreatedAt,
	).Scan(&banks.ID)

	if err != nil {
		t.log.Error(ctx, "BanksRepository.repository.InsertBanks.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertBanks.inserted_id", banks.ID)
   return banks.ID, nil

}
func (t BanksRepository)  UpdateBanks(ctx context.Context,banks *model.Banks, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("BanksRepository -> UpdateBanks", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateBanks")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateBanks.id", id)
	tracker.AddParam("repository.UpdateBanks.bankcodeinternal", banks.BankCodeInternal)

	banks.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_BANKS_UPDATE, 
			banks.BankCodeInternal,
			banks.BankCode,
			banks.Name,
			banks.FullName,
			banks.Website,
			banks.IsOpenFinance,
			banks.IsActive,
			banks.CreatedAt,
			banks.ID,
   )
	if err != nil {
		t.log.Error(ctx, "BanksRepository.repository.UpdateBanks.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateBanks.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateBanks.rows_affected", rowsAffected)
	return nil
}

