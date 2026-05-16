package user_bank_accountsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	user_bank_accountsRepo "palm-pay/app/user_bank_accounts/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type User_bank_accountsServiceIF interface {
     GetUser_bank_accounts(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_bank_accountsById(ctx context.Context, id int64) (*model.User_bank_accounts, error)
     GetUser_bank_accountsByBankAccountCode(ctx context.Context, bankaccountcode string) (*model.User_bank_accounts, error)
     InsertUser_bank_accounts(ctx context.Context, user_bank_accounts *model.User_bank_accounts) (int64, error)
     UpdateUser_bank_accounts(ctx context.Context, user_bank_accounts *model.User_bank_accounts, id int64) error
     DeleteUser_bank_accountsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     user_bank_accountsRepo user_bank_accountsRepo.User_bank_accountsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewUser_bank_accountsService(user_bank_accountsRepo user_bank_accountsRepo.User_bank_accountsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         user_bank_accountsRepo: user_bank_accountsRepo,
		  observability:  observabilidade.NewServiceObservability("service.user_bank_accounts"),
     }
}
func (r Resource)  GetUser_bank_accounts(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_bank_accountsService -> GetUser_bank_accounts", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_bank_accounts.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetUser_bank_accounts.offset", offset)
	tracker.AddParam("service.GetUser_bank_accounts.limit", limit)



	itemsPage, err = r.user_bank_accountsRepo.GetUser_bank_accounts(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.User_bank_accounts); ok {
		tracker.AddResult("service.GetUser_bank_accounts.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetUser_bank_accountsById(ctx context.Context, id int64) (user_bank_accounts *model.User_bank_accounts, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_bank_accountsService -> GetUser_bank_accountsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_bank_accountsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetUser_bank_accountsById.id", id)
	user_bank_accounts, err = r.user_bank_accountsRepo.GetUser_bank_accountsById(ctx, id)
	if err != nil {
		return user_bank_accounts, errors.New(app.MsgRepositoryError)
	}

	return user_bank_accounts, nil
}
func (r Resource)  GetUser_bank_accountsByBankAccountCode(ctx context.Context, bankaccountcode string) (user_bank_accounts *model.User_bank_accounts, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_bank_accountsService -> GetUser_bank_accountsByBankAccountCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetUser_bank_accountsByBankAccountCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetUser_bank_accountsByBankAccountCode.bankaccountcode", bankaccountcode)
	user_bank_accounts, err = r.user_bank_accountsRepo.GetUser_bank_accountsByBankAccountCode(ctx, bankaccountcode)
	if err != nil {
		return user_bank_accounts, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetUser_bank_accountsByBankAccountCode.found", user_bank_accounts != nil)
	return user_bank_accounts, nil
}
func (r Resource)  DeleteUser_bank_accountsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_bank_accountsService -> DeleteUser_bank_accountsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteUser_bank_accountsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteUser_bank_accountsById.id",id)

	result, err = r.user_bank_accountsRepo.DeleteUser_bank_accountsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteUser_bank_accountsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertUser_bank_accounts(ctx context.Context,user_bank_accounts *model.User_bank_accounts) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_bank_accountsService -> InsertUser_bank_accounts", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertUser_bank_accounts")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertUser_bank_accounts.bankaccountcode", user_bank_accounts.BankAccountCode)
	insertedId, err = r.user_bank_accountsRepo.InsertUser_bank_accounts(ctx, user_bank_accounts)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertUser_bank_accounts.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateUser_bank_accounts(ctx context.Context,user_bank_accounts *model.User_bank_accounts, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_bank_accountsService -> UpdateUser_bank_accounts", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateUser_bank_accounts")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateUser_bank_accounts.id", id)
	tracker.AddParam("service.UpdateUser_bank_accounts.bankaccountcode", user_bank_accounts.BankAccountCode)

	err = r.user_bank_accountsRepo.UpdateUser_bank_accounts(ctx, user_bank_accounts, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateUser_bank_accounts.updated", true)

	return nil
}

