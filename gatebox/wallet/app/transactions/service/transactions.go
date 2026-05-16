package transactionsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	transactionsRepo "palm-pay/app/transactions/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type TransactionsServiceIF interface {
     GetTransactions(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetTransactionsById(ctx context.Context, id int64) (*model.Transactions, error)
     GetTransactionsByTransactionCode(ctx context.Context, transactioncode string) (*model.Transactions, error)
     InsertTransactions(ctx context.Context, transactions *model.Transactions) (int64, error)
     UpdateTransactions(ctx context.Context, transactions *model.Transactions, id int64) error
     DeleteTransactionsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     transactionsRepo transactionsRepo.TransactionsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewTransactionsService(transactionsRepo transactionsRepo.TransactionsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         transactionsRepo: transactionsRepo,
		  observability:  observabilidade.NewServiceObservability("service.transactions"),
     }
}
func (r Resource)  GetTransactions(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("TransactionsService -> GetTransactions", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetTransactions.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetTransactions.offset", offset)
	tracker.AddParam("service.GetTransactions.limit", limit)



	itemsPage, err = r.transactionsRepo.GetTransactions(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Transactions); ok {
		tracker.AddResult("service.GetTransactions.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetTransactionsById(ctx context.Context, id int64) (transactions *model.Transactions, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("TransactionsService -> GetTransactionsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetTransactionsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetTransactionsById.id", id)
	transactions, err = r.transactionsRepo.GetTransactionsById(ctx, id)
	if err != nil {
		return transactions, errors.New(app.MsgRepositoryError)
	}

	return transactions, nil
}
func (r Resource)  GetTransactionsByTransactionCode(ctx context.Context, transactioncode string) (transactions *model.Transactions, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("TransactionsService -> GetTransactionsByTransactionCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetTransactionsByTransactionCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetTransactionsByTransactionCode.transactioncode", transactioncode)
	transactions, err = r.transactionsRepo.GetTransactionsByTransactionCode(ctx, transactioncode)
	if err != nil {
		return transactions, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetTransactionsByTransactionCode.found", transactions != nil)
	return transactions, nil
}
func (r Resource)  DeleteTransactionsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("TransactionsService -> DeleteTransactionsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteTransactionsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteTransactionsById.id",id)

	result, err = r.transactionsRepo.DeleteTransactionsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteTransactionsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertTransactions(ctx context.Context,transactions *model.Transactions) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("TransactionsService -> InsertTransactions", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertTransactions")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertTransactions.transactioncode", transactions.TransactionCode)
	insertedId, err = r.transactionsRepo.InsertTransactions(ctx, transactions)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertTransactions.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateTransactions(ctx context.Context,transactions *model.Transactions, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("TransactionsService -> UpdateTransactions", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateTransactions")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateTransactions.id", id)
	tracker.AddParam("service.UpdateTransactions.transactioncode", transactions.TransactionCode)

	err = r.transactionsRepo.UpdateTransactions(ctx, transactions, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateTransactions.updated", true)

	return nil
}

