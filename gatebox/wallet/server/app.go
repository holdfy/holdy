package server

import (
     "context"
     "fmt"
     "net/http"
     "os"
     "os/signal"
     "syscall"
     "time"
     "github.com/jackc/pgx/v5/pgxpool"
     "github.com/labstack/echo/v4"
	  "github.com/labstack/echo/v4/middleware"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
	 "github.com/labstack/gommon/log"

    user_restrictionsHandler "palm-pay/app/user_restrictions/handler"
    user_restrictionsRepo "palm-pay/app/user_restrictions/repository"
    user_restrictionsSV "palm-pay/app/user_restrictions/service"

    restriction_typesHandler "palm-pay/app/restriction_types/handler"
    restriction_typesRepo "palm-pay/app/restriction_types/repository"
    restriction_typesSV "palm-pay/app/restriction_types/service"

    security_eventsHandler "palm-pay/app/security_events/handler"
    security_eventsRepo "palm-pay/app/security_events/repository"
    security_eventsSV "palm-pay/app/security_events/service"

    security_severity_levelsHandler "palm-pay/app/security_severity_levels/handler"
    security_severity_levelsRepo "palm-pay/app/security_severity_levels/repository"
    security_severity_levelsSV "palm-pay/app/security_severity_levels/service"

    security_event_typesHandler "palm-pay/app/security_event_types/handler"
    security_event_typesRepo "palm-pay/app/security_event_types/repository"
    security_event_typesSV "palm-pay/app/security_event_types/service"

    user_sessionsHandler "palm-pay/app/user_sessions/handler"
    user_sessionsRepo "palm-pay/app/user_sessions/repository"
    user_sessionsSV "palm-pay/app/user_sessions/service"

    device_typesHandler "palm-pay/app/device_types/handler"
    device_typesRepo "palm-pay/app/device_types/repository"
    device_typesSV "palm-pay/app/device_types/service"

    session_statusHandler "palm-pay/app/session_status/handler"
    session_statusRepo "palm-pay/app/session_status/repository"
    session_statusSV "palm-pay/app/session_status/service"

    audit_logHandler "palm-pay/app/audit_log/handler"
    audit_logRepo "palm-pay/app/audit_log/repository"
    audit_logSV "palm-pay/app/audit_log/service"

    audit_tablesHandler "palm-pay/app/audit_tables/handler"
    audit_tablesRepo "palm-pay/app/audit_tables/repository"
    audit_tablesSV "palm-pay/app/audit_tables/service"

    audit_actionsHandler "palm-pay/app/audit_actions/handler"
    audit_actionsRepo "palm-pay/app/audit_actions/repository"
    audit_actionsSV "palm-pay/app/audit_actions/service"

    notification_historyHandler "palm-pay/app/notification_history/handler"
    notification_historyRepo "palm-pay/app/notification_history/repository"
    notification_historySV "palm-pay/app/notification_history/service"

    notification_statusHandler "palm-pay/app/notification_status/handler"
    notification_statusRepo "palm-pay/app/notification_status/repository"
    notification_statusSV "palm-pay/app/notification_status/service"

    notification_templatesHandler "palm-pay/app/notification_templates/handler"
    notification_templatesRepo "palm-pay/app/notification_templates/repository"
    notification_templatesSV "palm-pay/app/notification_templates/service"

    notification_channelsHandler "palm-pay/app/notification_channels/handler"
    notification_channelsRepo "palm-pay/app/notification_channels/repository"
    notification_channelsSV "palm-pay/app/notification_channels/service"

    gateway_transactionsHandler "palm-pay/app/gateway_transactions/handler"
    gateway_transactionsRepo "palm-pay/app/gateway_transactions/repository"
    gateway_transactionsSV "palm-pay/app/gateway_transactions/service"

    gateway_status_typesHandler "palm-pay/app/gateway_status_types/handler"
    gateway_status_typesRepo "palm-pay/app/gateway_status_types/repository"
    gateway_status_typesSV "palm-pay/app/gateway_status_types/service"

    gatewaysHandler "palm-pay/app/gateways/handler"
    gatewaysRepo "palm-pay/app/gateways/repository"
    gatewaysSV "palm-pay/app/gateways/service"

    transaction_status_historyHandler "palm-pay/app/transaction_status_history/handler"
    transaction_status_historyRepo "palm-pay/app/transaction_status_history/repository"
    transaction_status_historySV "palm-pay/app/transaction_status_history/service"

    change_actorsHandler "palm-pay/app/change_actors/handler"
    change_actorsRepo "palm-pay/app/change_actors/repository"
    change_actorsSV "palm-pay/app/change_actors/service"

    transactionsHandler "palm-pay/app/transactions/handler"
    transactionsRepo "palm-pay/app/transactions/repository"
    transactionsSV "palm-pay/app/transactions/service"

    transaction_statusHandler "palm-pay/app/transaction_status/handler"
    transaction_statusRepo "palm-pay/app/transaction_status/repository"
    transaction_statusSV "palm-pay/app/transaction_status/service"

    signature_methodsHandler "palm-pay/app/signature_methods/handler"
    signature_methodsRepo "palm-pay/app/signature_methods/repository"
    signature_methodsSV "palm-pay/app/signature_methods/service"

    payment_methodsHandler "palm-pay/app/payment_methods/handler"
    payment_methodsRepo "palm-pay/app/payment_methods/repository"
    payment_methodsSV "palm-pay/app/payment_methods/service"

    transaction_typesHandler "palm-pay/app/transaction_types/handler"
    transaction_typesRepo "palm-pay/app/transaction_types/repository"
    transaction_typesSV "palm-pay/app/transaction_types/service"

    external_walletsHandler "palm-pay/app/external_wallets/handler"
    external_walletsRepo "palm-pay/app/external_wallets/repository"
    external_walletsSV "palm-pay/app/external_wallets/service"

    wallet_providersHandler "palm-pay/app/wallet_providers/handler"
    wallet_providersRepo "palm-pay/app/wallet_providers/repository"
    wallet_providersSV "palm-pay/app/wallet_providers/service"

    user_bank_accountsHandler "palm-pay/app/user_bank_accounts/handler"
    user_bank_accountsRepo "palm-pay/app/user_bank_accounts/repository"
    user_bank_accountsSV "palm-pay/app/user_bank_accounts/service"

    account_typesHandler "palm-pay/app/account_types/handler"
    account_typesRepo "palm-pay/app/account_types/repository"
    account_typesSV "palm-pay/app/account_types/service"

    banksHandler "palm-pay/app/banks/handler"
    banksRepo "palm-pay/app/banks/repository"
    banksSV "palm-pay/app/banks/service"

    user_cardsHandler "palm-pay/app/user_cards/handler"
    user_cardsRepo "palm-pay/app/user_cards/repository"
    user_cardsSV "palm-pay/app/user_cards/service"

    acquirersHandler "palm-pay/app/acquirers/handler"
    acquirersRepo "palm-pay/app/acquirers/repository"
    acquirersSV "palm-pay/app/acquirers/service"

    card_typesHandler "palm-pay/app/card_types/handler"
    card_typesRepo "palm-pay/app/card_types/repository"
    card_typesSV "palm-pay/app/card_types/service"

    card_brandsHandler "palm-pay/app/card_brands/handler"
    card_brandsRepo "palm-pay/app/card_brands/repository"
    card_brandsSV "palm-pay/app/card_brands/service"

    wallet_balance_historyHandler "palm-pay/app/wallet_balance_history/handler"
    wallet_balance_historyRepo "palm-pay/app/wallet_balance_history/repository"
    wallet_balance_historySV "palm-pay/app/wallet_balance_history/service"

    balance_change_typesHandler "palm-pay/app/balance_change_types/handler"
    balance_change_typesRepo "palm-pay/app/balance_change_types/repository"
    balance_change_typesSV "palm-pay/app/balance_change_types/service"

    walletsHandler "palm-pay/app/wallets/handler"
    walletsRepo "palm-pay/app/wallets/repository"
    walletsSV "palm-pay/app/wallets/service"

    currenciesHandler "palm-pay/app/currencies/handler"
    currenciesRepo "palm-pay/app/currencies/repository"
    currenciesSV "palm-pay/app/currencies/service"

    wallet_statusHandler "palm-pay/app/wallet_status/handler"
    wallet_statusRepo "palm-pay/app/wallet_status/repository"
    wallet_statusSV "palm-pay/app/wallet_status/service"

    wallet_typesHandler "palm-pay/app/wallet_types/handler"
    wallet_typesRepo "palm-pay/app/wallet_types/repository"
    wallet_typesSV "palm-pay/app/wallet_types/service"

    biometric_attemptsHandler "palm-pay/app/biometric_attempts/handler"
    biometric_attemptsRepo "palm-pay/app/biometric_attempts/repository"
    biometric_attemptsSV "palm-pay/app/biometric_attempts/service"

    failure_reasonsHandler "palm-pay/app/failure_reasons/handler"
    failure_reasonsRepo "palm-pay/app/failure_reasons/repository"
    failure_reasonsSV "palm-pay/app/failure_reasons/service"

    attempt_resultsHandler "palm-pay/app/attempt_results/handler"
    attempt_resultsRepo "palm-pay/app/attempt_results/repository"
    attempt_resultsSV "palm-pay/app/attempt_results/service"

    palm_biometricsHandler "palm-pay/app/palm_biometrics/handler"
    palm_biometricsRepo "palm-pay/app/palm_biometrics/repository"
    palm_biometricsSV "palm-pay/app/palm_biometrics/service"

    hand_typesHandler "palm-pay/app/hand_types/handler"
    hand_typesRepo "palm-pay/app/hand_types/repository"
    hand_typesSV "palm-pay/app/hand_types/service"

    user_addressesHandler "palm-pay/app/user_addresses/handler"
    user_addressesRepo "palm-pay/app/user_addresses/repository"
    user_addressesSV "palm-pay/app/user_addresses/service"

    address_typesHandler "palm-pay/app/address_types/handler"
    address_typesRepo "palm-pay/app/address_types/repository"
    address_typesSV "palm-pay/app/address_types/service"

    user_documentsHandler "palm-pay/app/user_documents/handler"
    user_documentsRepo "palm-pay/app/user_documents/repository"
    user_documentsSV "palm-pay/app/user_documents/service"

    document_statusHandler "palm-pay/app/document_status/handler"
    document_statusRepo "palm-pay/app/document_status/repository"
    document_statusSV "palm-pay/app/document_status/service"

    document_typesHandler "palm-pay/app/document_types/handler"
    document_typesRepo "palm-pay/app/document_types/repository"
    document_typesSV "palm-pay/app/document_types/service"

    usersHandler "palm-pay/app/users/handler"
    usersRepo "palm-pay/app/users/repository"
    usersSV "palm-pay/app/users/service"

    kyc_statusHandler "palm-pay/app/kyc_status/handler"
    kyc_statusRepo "palm-pay/app/kyc_status/repository"
    kyc_statusSV "palm-pay/app/kyc_status/service"

    user_statusHandler "palm-pay/app/user_status/handler"
    user_statusRepo "palm-pay/app/user_status/repository"
    user_statusSV "palm-pay/app/user_status/service"

    system_configurationsHandler "palm-pay/app/system_configurations/handler"
    system_configurationsRepo "palm-pay/app/system_configurations/repository"
    system_configurationsSV "palm-pay/app/system_configurations/service"

    applicationsHandler "palm-pay/app/applications/handler"
    applicationsRepo "palm-pay/app/applications/repository"
    applicationsSV "palm-pay/app/applications/service"

    "github.com/tungstenbyte/utils/logger"
)

type App struct {
	httpServer        *http.Server
	PGDBWrite         *pgxpool.Pool
	PGDBRead          *pgxpool.Pool
	log logger.Logger
	// healthUC          healthUC.HealthUseCaseIF
    user_restrictionsSV user_restrictionsSV.User_restrictionsServiceIF
    restriction_typesSV restriction_typesSV.Restriction_typesServiceIF
    security_eventsSV security_eventsSV.Security_eventsServiceIF
    security_severity_levelsSV security_severity_levelsSV.Security_severity_levelsServiceIF
    security_event_typesSV security_event_typesSV.Security_event_typesServiceIF
    user_sessionsSV user_sessionsSV.User_sessionsServiceIF
    device_typesSV device_typesSV.Device_typesServiceIF
    session_statusSV session_statusSV.Session_statusServiceIF
    audit_logSV audit_logSV.Audit_logServiceIF
    audit_tablesSV audit_tablesSV.Audit_tablesServiceIF
    audit_actionsSV audit_actionsSV.Audit_actionsServiceIF
    notification_historySV notification_historySV.Notification_historyServiceIF
    notification_statusSV notification_statusSV.Notification_statusServiceIF
    notification_templatesSV notification_templatesSV.Notification_templatesServiceIF
    notification_channelsSV notification_channelsSV.Notification_channelsServiceIF
    gateway_transactionsSV gateway_transactionsSV.Gateway_transactionsServiceIF
    gateway_status_typesSV gateway_status_typesSV.Gateway_status_typesServiceIF
    gatewaysSV gatewaysSV.GatewaysServiceIF
    transaction_status_historySV transaction_status_historySV.Transaction_status_historyServiceIF
    change_actorsSV change_actorsSV.Change_actorsServiceIF
    transactionsSV transactionsSV.TransactionsServiceIF
    transaction_statusSV transaction_statusSV.Transaction_statusServiceIF
    signature_methodsSV signature_methodsSV.Signature_methodsServiceIF
    payment_methodsSV payment_methodsSV.Payment_methodsServiceIF
    transaction_typesSV transaction_typesSV.Transaction_typesServiceIF
    external_walletsSV external_walletsSV.External_walletsServiceIF
    wallet_providersSV wallet_providersSV.Wallet_providersServiceIF
    user_bank_accountsSV user_bank_accountsSV.User_bank_accountsServiceIF
    account_typesSV account_typesSV.Account_typesServiceIF
    banksSV banksSV.BanksServiceIF
    user_cardsSV user_cardsSV.User_cardsServiceIF
    acquirersSV acquirersSV.AcquirersServiceIF
    card_typesSV card_typesSV.Card_typesServiceIF
    card_brandsSV card_brandsSV.Card_brandsServiceIF
    wallet_balance_historySV wallet_balance_historySV.Wallet_balance_historyServiceIF
    balance_change_typesSV balance_change_typesSV.Balance_change_typesServiceIF
    walletsSV walletsSV.WalletsServiceIF
    currenciesSV currenciesSV.CurrenciesServiceIF
    wallet_statusSV wallet_statusSV.Wallet_statusServiceIF
    wallet_typesSV wallet_typesSV.Wallet_typesServiceIF
    biometric_attemptsSV biometric_attemptsSV.Biometric_attemptsServiceIF
    failure_reasonsSV failure_reasonsSV.Failure_reasonsServiceIF
    attempt_resultsSV attempt_resultsSV.Attempt_resultsServiceIF
    palm_biometricsSV palm_biometricsSV.Palm_biometricsServiceIF
    hand_typesSV hand_typesSV.Hand_typesServiceIF
    user_addressesSV user_addressesSV.User_addressesServiceIF
    address_typesSV address_typesSV.Address_typesServiceIF
    user_documentsSV user_documentsSV.User_documentsServiceIF
    document_statusSV document_statusSV.Document_statusServiceIF
    document_typesSV document_typesSV.Document_typesServiceIF
    usersSV usersSV.UsersServiceIF
    kyc_statusSV kyc_statusSV.Kyc_statusServiceIF
    user_statusSV user_statusSV.User_statusServiceIF
    system_configurationsSV system_configurationsSV.System_configurationsServiceIF
    applicationsSV applicationsSV.ApplicationsServiceIF
}

type ServerIF interface {
	Start()
	Stop()
	Run(port string) error
}

func New() ServerIF {
	return &App{}
}

func (a *App) Start() {

	a.log = logger.NewApiLogger()
	a.log.InitLogger("Dpanic")
	a.StartPGWrite()
	a.StartPGRead()
	// a.StartRedisCluster()

    user_restrictionsRepository  := user_restrictionsRepo.NewUser_restrictionsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.user_restrictionsSV = user_restrictionsSV.NewUser_restrictionsService(user_restrictionsRepository, a.log)

    restriction_typesRepository  := restriction_typesRepo.NewRestriction_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.restriction_typesSV = restriction_typesSV.NewRestriction_typesService(restriction_typesRepository, a.log)

    security_eventsRepository  := security_eventsRepo.NewSecurity_eventsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.security_eventsSV = security_eventsSV.NewSecurity_eventsService(security_eventsRepository, a.log)

    security_severity_levelsRepository  := security_severity_levelsRepo.NewSecurity_severity_levelsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.security_severity_levelsSV = security_severity_levelsSV.NewSecurity_severity_levelsService(security_severity_levelsRepository, a.log)

    security_event_typesRepository  := security_event_typesRepo.NewSecurity_event_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.security_event_typesSV = security_event_typesSV.NewSecurity_event_typesService(security_event_typesRepository, a.log)

    user_sessionsRepository  := user_sessionsRepo.NewUser_sessionsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.user_sessionsSV = user_sessionsSV.NewUser_sessionsService(user_sessionsRepository, a.log)

    device_typesRepository  := device_typesRepo.NewDevice_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.device_typesSV = device_typesSV.NewDevice_typesService(device_typesRepository, a.log)

    session_statusRepository  := session_statusRepo.NewSession_statusRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.session_statusSV = session_statusSV.NewSession_statusService(session_statusRepository, a.log)

    audit_logRepository  := audit_logRepo.NewAudit_logRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.audit_logSV = audit_logSV.NewAudit_logService(audit_logRepository, a.log)

    audit_tablesRepository  := audit_tablesRepo.NewAudit_tablesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.audit_tablesSV = audit_tablesSV.NewAudit_tablesService(audit_tablesRepository, a.log)

    audit_actionsRepository  := audit_actionsRepo.NewAudit_actionsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.audit_actionsSV = audit_actionsSV.NewAudit_actionsService(audit_actionsRepository, a.log)

    notification_historyRepository  := notification_historyRepo.NewNotification_historyRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.notification_historySV = notification_historySV.NewNotification_historyService(notification_historyRepository, a.log)

    notification_statusRepository  := notification_statusRepo.NewNotification_statusRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.notification_statusSV = notification_statusSV.NewNotification_statusService(notification_statusRepository, a.log)

    notification_templatesRepository  := notification_templatesRepo.NewNotification_templatesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.notification_templatesSV = notification_templatesSV.NewNotification_templatesService(notification_templatesRepository, a.log)

    notification_channelsRepository  := notification_channelsRepo.NewNotification_channelsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.notification_channelsSV = notification_channelsSV.NewNotification_channelsService(notification_channelsRepository, a.log)

    gateway_transactionsRepository  := gateway_transactionsRepo.NewGateway_transactionsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.gateway_transactionsSV = gateway_transactionsSV.NewGateway_transactionsService(gateway_transactionsRepository, a.log)

    gateway_status_typesRepository  := gateway_status_typesRepo.NewGateway_status_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.gateway_status_typesSV = gateway_status_typesSV.NewGateway_status_typesService(gateway_status_typesRepository, a.log)

    gatewaysRepository  := gatewaysRepo.NewGatewaysRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.gatewaysSV = gatewaysSV.NewGatewaysService(gatewaysRepository, a.log)

    transaction_status_historyRepository  := transaction_status_historyRepo.NewTransaction_status_historyRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.transaction_status_historySV = transaction_status_historySV.NewTransaction_status_historyService(transaction_status_historyRepository, a.log)

    change_actorsRepository  := change_actorsRepo.NewChange_actorsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.change_actorsSV = change_actorsSV.NewChange_actorsService(change_actorsRepository, a.log)

    transactionsRepository  := transactionsRepo.NewTransactionsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.transactionsSV = transactionsSV.NewTransactionsService(transactionsRepository, a.log)

    transaction_statusRepository  := transaction_statusRepo.NewTransaction_statusRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.transaction_statusSV = transaction_statusSV.NewTransaction_statusService(transaction_statusRepository, a.log)

    signature_methodsRepository  := signature_methodsRepo.NewSignature_methodsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.signature_methodsSV = signature_methodsSV.NewSignature_methodsService(signature_methodsRepository, a.log)

    payment_methodsRepository  := payment_methodsRepo.NewPayment_methodsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.payment_methodsSV = payment_methodsSV.NewPayment_methodsService(payment_methodsRepository, a.log)

    transaction_typesRepository  := transaction_typesRepo.NewTransaction_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.transaction_typesSV = transaction_typesSV.NewTransaction_typesService(transaction_typesRepository, a.log)

    external_walletsRepository  := external_walletsRepo.NewExternal_walletsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.external_walletsSV = external_walletsSV.NewExternal_walletsService(external_walletsRepository, a.log)

    wallet_providersRepository  := wallet_providersRepo.NewWallet_providersRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.wallet_providersSV = wallet_providersSV.NewWallet_providersService(wallet_providersRepository, a.log)

    user_bank_accountsRepository  := user_bank_accountsRepo.NewUser_bank_accountsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.user_bank_accountsSV = user_bank_accountsSV.NewUser_bank_accountsService(user_bank_accountsRepository, a.log)

    account_typesRepository  := account_typesRepo.NewAccount_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.account_typesSV = account_typesSV.NewAccount_typesService(account_typesRepository, a.log)

    banksRepository  := banksRepo.NewBanksRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.banksSV = banksSV.NewBanksService(banksRepository, a.log)

    user_cardsRepository  := user_cardsRepo.NewUser_cardsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.user_cardsSV = user_cardsSV.NewUser_cardsService(user_cardsRepository, a.log)

    acquirersRepository  := acquirersRepo.NewAcquirersRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.acquirersSV = acquirersSV.NewAcquirersService(acquirersRepository, a.log)

    card_typesRepository  := card_typesRepo.NewCard_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.card_typesSV = card_typesSV.NewCard_typesService(card_typesRepository, a.log)

    card_brandsRepository  := card_brandsRepo.NewCard_brandsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.card_brandsSV = card_brandsSV.NewCard_brandsService(card_brandsRepository, a.log)

    wallet_balance_historyRepository  := wallet_balance_historyRepo.NewWallet_balance_historyRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.wallet_balance_historySV = wallet_balance_historySV.NewWallet_balance_historyService(wallet_balance_historyRepository, a.log)

    balance_change_typesRepository  := balance_change_typesRepo.NewBalance_change_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.balance_change_typesSV = balance_change_typesSV.NewBalance_change_typesService(balance_change_typesRepository, a.log)

    walletsRepository  := walletsRepo.NewWalletsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.walletsSV = walletsSV.NewWalletsService(walletsRepository, a.log)

    currenciesRepository  := currenciesRepo.NewCurrenciesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.currenciesSV = currenciesSV.NewCurrenciesService(currenciesRepository, a.log)

    wallet_statusRepository  := wallet_statusRepo.NewWallet_statusRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.wallet_statusSV = wallet_statusSV.NewWallet_statusService(wallet_statusRepository, a.log)

    wallet_typesRepository  := wallet_typesRepo.NewWallet_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.wallet_typesSV = wallet_typesSV.NewWallet_typesService(wallet_typesRepository, a.log)

    biometric_attemptsRepository  := biometric_attemptsRepo.NewBiometric_attemptsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.biometric_attemptsSV = biometric_attemptsSV.NewBiometric_attemptsService(biometric_attemptsRepository, a.log)

    failure_reasonsRepository  := failure_reasonsRepo.NewFailure_reasonsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.failure_reasonsSV = failure_reasonsSV.NewFailure_reasonsService(failure_reasonsRepository, a.log)

    attempt_resultsRepository  := attempt_resultsRepo.NewAttempt_resultsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.attempt_resultsSV = attempt_resultsSV.NewAttempt_resultsService(attempt_resultsRepository, a.log)

    palm_biometricsRepository  := palm_biometricsRepo.NewPalm_biometricsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.palm_biometricsSV = palm_biometricsSV.NewPalm_biometricsService(palm_biometricsRepository, a.log)

    hand_typesRepository  := hand_typesRepo.NewHand_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.hand_typesSV = hand_typesSV.NewHand_typesService(hand_typesRepository, a.log)

    user_addressesRepository  := user_addressesRepo.NewUser_addressesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.user_addressesSV = user_addressesSV.NewUser_addressesService(user_addressesRepository, a.log)

    address_typesRepository  := address_typesRepo.NewAddress_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.address_typesSV = address_typesSV.NewAddress_typesService(address_typesRepository, a.log)

    user_documentsRepository  := user_documentsRepo.NewUser_documentsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.user_documentsSV = user_documentsSV.NewUser_documentsService(user_documentsRepository, a.log)

    document_statusRepository  := document_statusRepo.NewDocument_statusRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.document_statusSV = document_statusSV.NewDocument_statusService(document_statusRepository, a.log)

    document_typesRepository  := document_typesRepo.NewDocument_typesRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.document_typesSV = document_typesSV.NewDocument_typesService(document_typesRepository, a.log)

    usersRepository  := usersRepo.NewUsersRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.usersSV = usersSV.NewUsersService(usersRepository, a.log)

    kyc_statusRepository  := kyc_statusRepo.NewKyc_statusRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.kyc_statusSV = kyc_statusSV.NewKyc_statusService(kyc_statusRepository, a.log)

    user_statusRepository  := user_statusRepo.NewUser_statusRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.user_statusSV = user_statusSV.NewUser_statusService(user_statusRepository, a.log)

    system_configurationsRepository  := system_configurationsRepo.NewSystem_configurationsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.system_configurationsSV = system_configurationsSV.NewSystem_configurationsService(system_configurationsRepository, a.log)

    applicationsRepository  := applicationsRepo.NewApplicationsRepository(a.PGDBWrite, a.PGDBRead, a.log)
    a.applicationsSV = applicationsSV.NewApplicationsService(applicationsRepository, a.log)




   // healthRepository := healthRepo.NewHealthRepository(a.PGDBRead, a.log)
	// a.healthUC = healthUC.NewHealthUseCase(healthRepository, a.log)
}

func (a *App) Run(port string) error {
	var (
		sig chan os.Signal
	)
	router := echo.New()
	router.HideBanner = true
	router.HidePort = true
	router.Server.ReadTimeout = 15 * time.Minute

	router.Use()
	router.Use(middleware.Recover())

	// <---- adicionado aqui: Middlewares de observabilidade em ordem específica
	router.Use(observabilidade.RequestIDMiddleware())                   // 1º - Request ID sempre primeiro
	router.Use(observabilidade.TimeoutMiddleware(30 * time.Second))     // 2º - Timeout padrão
	router.Use(observabilidade.StructuredLoggingMiddleware("global"))   // 3º - Log estruturado
	router.Use(middleware.Recover())                                    // 4º - Recovery do Echo
	router.Use(observabilidade.EnhancedHTTPMetricsMiddleware("global")) // 5º - Métricas HTTP

	// <---- adicionado aqui: Endpoint de health check com observabilidade
	router.GET("/health", func(c echo.Context) error {
		// <---- adicionado aqui: Health check simples com métricas
		start := time.Now()

		// Verificar conexões do banco
		ctxDB, cancel := context.WithTimeout(c.Request().Context(), 2*time.Second)
		defer cancel()

		health := map[string]interface{}{
			"status":    "healthy",
			"timestamp": time.Now().Format(time.RFC3339),
			"version":   "1.0.0",
			"service":   "novo-exemplo-palm-pay",
		}

		// Verificar PG Write
		if err := a.PGDBWrite.Ping(ctxDB); err != nil {
			health["status"] = "unhealthy"
			health["pg_write"] = "error: " + err.Error()
		} else {
			health["pg_write"] = "ok"
		}

		// Verificar PG Read
		if err := a.PGDBRead.Ping(ctxDB); err != nil {
			health["status"] = "unhealthy"
			health["pg_read"] = "error: " + err.Error()
		} else {
			health["pg_read"] = "ok"
		}

		health["response_time_ms"] = time.Since(start).Milliseconds()

		if health["status"] == "healthy" {
			return c.JSON(http.StatusOK, health)
		} else {
			return c.JSON(http.StatusServiceUnavailable, health)
		}
	})

	// <---- adicionado aqui: Endpoint de métricas interno (além do servidor separado)
	router.GET("/internal/metrics", func(c echo.Context) error {
		return c.JSON(http.StatusOK, map[string]interface{}{
			"metrics_endpoint":  "http://localhost:2112/metrics",
			"prometheus_format": true,
			"service":           "novo-exemplo-palm-pay",
		})
	})

   api := router.Group("/api")

	// health.RegisterHTTPEndpoints(api, a.healthUC, a.log)

	user_restrictionsHandler.RegisterUser_restrictionsHTTPEndpoints(api, a.user_restrictionsSV, a.log)
	restriction_typesHandler.RegisterRestriction_typesHTTPEndpoints(api, a.restriction_typesSV, a.log)
	security_eventsHandler.RegisterSecurity_eventsHTTPEndpoints(api, a.security_eventsSV, a.log)
	security_severity_levelsHandler.RegisterSecurity_severity_levelsHTTPEndpoints(api, a.security_severity_levelsSV, a.log)
	security_event_typesHandler.RegisterSecurity_event_typesHTTPEndpoints(api, a.security_event_typesSV, a.log)
	user_sessionsHandler.RegisterUser_sessionsHTTPEndpoints(api, a.user_sessionsSV, a.log)
	device_typesHandler.RegisterDevice_typesHTTPEndpoints(api, a.device_typesSV, a.log)
	session_statusHandler.RegisterSession_statusHTTPEndpoints(api, a.session_statusSV, a.log)
	audit_logHandler.RegisterAudit_logHTTPEndpoints(api, a.audit_logSV, a.log)
	audit_tablesHandler.RegisterAudit_tablesHTTPEndpoints(api, a.audit_tablesSV, a.log)
	audit_actionsHandler.RegisterAudit_actionsHTTPEndpoints(api, a.audit_actionsSV, a.log)
	notification_historyHandler.RegisterNotification_historyHTTPEndpoints(api, a.notification_historySV, a.log)
	notification_statusHandler.RegisterNotification_statusHTTPEndpoints(api, a.notification_statusSV, a.log)
	notification_templatesHandler.RegisterNotification_templatesHTTPEndpoints(api, a.notification_templatesSV, a.log)
	notification_channelsHandler.RegisterNotification_channelsHTTPEndpoints(api, a.notification_channelsSV, a.log)
	gateway_transactionsHandler.RegisterGateway_transactionsHTTPEndpoints(api, a.gateway_transactionsSV, a.log)
	gateway_status_typesHandler.RegisterGateway_status_typesHTTPEndpoints(api, a.gateway_status_typesSV, a.log)
	gatewaysHandler.RegisterGatewaysHTTPEndpoints(api, a.gatewaysSV, a.log)
	transaction_status_historyHandler.RegisterTransaction_status_historyHTTPEndpoints(api, a.transaction_status_historySV, a.log)
	change_actorsHandler.RegisterChange_actorsHTTPEndpoints(api, a.change_actorsSV, a.log)
	transactionsHandler.RegisterTransactionsHTTPEndpoints(api, a.transactionsSV, a.log)
	transaction_statusHandler.RegisterTransaction_statusHTTPEndpoints(api, a.transaction_statusSV, a.log)
	signature_methodsHandler.RegisterSignature_methodsHTTPEndpoints(api, a.signature_methodsSV, a.log)
	payment_methodsHandler.RegisterPayment_methodsHTTPEndpoints(api, a.payment_methodsSV, a.log)
	transaction_typesHandler.RegisterTransaction_typesHTTPEndpoints(api, a.transaction_typesSV, a.log)
	external_walletsHandler.RegisterExternal_walletsHTTPEndpoints(api, a.external_walletsSV, a.log)
	wallet_providersHandler.RegisterWallet_providersHTTPEndpoints(api, a.wallet_providersSV, a.log)
	user_bank_accountsHandler.RegisterUser_bank_accountsHTTPEndpoints(api, a.user_bank_accountsSV, a.log)
	account_typesHandler.RegisterAccount_typesHTTPEndpoints(api, a.account_typesSV, a.log)
	banksHandler.RegisterBanksHTTPEndpoints(api, a.banksSV, a.log)
	user_cardsHandler.RegisterUser_cardsHTTPEndpoints(api, a.user_cardsSV, a.log)
	acquirersHandler.RegisterAcquirersHTTPEndpoints(api, a.acquirersSV, a.log)
	card_typesHandler.RegisterCard_typesHTTPEndpoints(api, a.card_typesSV, a.log)
	card_brandsHandler.RegisterCard_brandsHTTPEndpoints(api, a.card_brandsSV, a.log)
	wallet_balance_historyHandler.RegisterWallet_balance_historyHTTPEndpoints(api, a.wallet_balance_historySV, a.log)
	balance_change_typesHandler.RegisterBalance_change_typesHTTPEndpoints(api, a.balance_change_typesSV, a.log)
	walletsHandler.RegisterWalletsHTTPEndpoints(api, a.walletsSV, a.log)
	currenciesHandler.RegisterCurrenciesHTTPEndpoints(api, a.currenciesSV, a.log)
	wallet_statusHandler.RegisterWallet_statusHTTPEndpoints(api, a.wallet_statusSV, a.log)
	wallet_typesHandler.RegisterWallet_typesHTTPEndpoints(api, a.wallet_typesSV, a.log)
	biometric_attemptsHandler.RegisterBiometric_attemptsHTTPEndpoints(api, a.biometric_attemptsSV, a.log)
	failure_reasonsHandler.RegisterFailure_reasonsHTTPEndpoints(api, a.failure_reasonsSV, a.log)
	attempt_resultsHandler.RegisterAttempt_resultsHTTPEndpoints(api, a.attempt_resultsSV, a.log)
	palm_biometricsHandler.RegisterPalm_biometricsHTTPEndpoints(api, a.palm_biometricsSV, a.log)
	hand_typesHandler.RegisterHand_typesHTTPEndpoints(api, a.hand_typesSV, a.log)
	user_addressesHandler.RegisterUser_addressesHTTPEndpoints(api, a.user_addressesSV, a.log)
	address_typesHandler.RegisterAddress_typesHTTPEndpoints(api, a.address_typesSV, a.log)
	user_documentsHandler.RegisterUser_documentsHTTPEndpoints(api, a.user_documentsSV, a.log)
	document_statusHandler.RegisterDocument_statusHTTPEndpoints(api, a.document_statusSV, a.log)
	document_typesHandler.RegisterDocument_typesHTTPEndpoints(api, a.document_typesSV, a.log)
	usersHandler.RegisterUsersHTTPEndpoints(api, a.usersSV, a.log)
	kyc_statusHandler.RegisterKyc_statusHTTPEndpoints(api, a.kyc_statusSV, a.log)
	user_statusHandler.RegisterUser_statusHTTPEndpoints(api, a.user_statusSV, a.log)
	system_configurationsHandler.RegisterSystem_configurationsHTTPEndpoints(api, a.system_configurationsSV, a.log)
	applicationsHandler.RegisterApplicationsHTTPEndpoints(api, a.applicationsSV, a.log)

	a.httpServer = &http.Server{
		Addr:           ":" + port,
		Handler:        router,
		ReadTimeout:    10 * time.Second,
		WriteTimeout:   10 * time.Second,
		MaxHeaderBytes: 1 << 20,
	}

	fmt.Println("≡ Microservices palm-pay Started in local Port: ", port, " ≡")
	fmt.Println("📊 Métricas disponíveis em: http://localhost:2112/metrics") // <---- adicionado aqui
	fmt.Println("🔍 Health check em: http://localhost:" + port + "/health")  // <---- adicionado aqui
	fmt.Println("")

	go func() {
		if err := a.httpServer.ListenAndServe(); err != nil {
			log.Fatalf("Failed to listen and serve: %+v", err)
		}
	}()

	quit1 := make(chan os.Signal, 1)
	// signal.Notify(quit, os.Interrupt, os.Interrupt)
	sig = make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGTERM, syscall.SIGHUP)

	<-quit1

	ctx, shutdown := context.WithTimeout(context.Background(), 5*time.Second)
	defer shutdown()
	return a.httpServer.Shutdown(ctx)
}

func (a *App) StartPGWrite() {
	var err error
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	url := os.Getenv("POSTGRESQL_WRITE_URL")

	config, err := pgxpool.ParseConfig(url) 
	if err != nil {
		log.Fatal(ctx, "Failed to parse PostgreSQL config: ", err.Error())
	}

	config.MaxConns = 30                      
	config.MinConns = 5                       
	config.MaxConnLifetime = time.Hour        
	config.MaxConnIdleTime = 30 * time.Minute

	a.PGDBWrite, err = pgxpool.NewWithConfig(ctx, config)

	if err != nil {
		log.Fatal(ctx, "Not connect DB Postgresql: ", err.Error())
	}

	if err := a.PGDBWrite.Ping(ctx); err != nil {
		log.Fatal(ctx, "Failed to ping PostgreSQL Write: ", err.Error())
	}

	a.log.Info("DB Postgresql Writer was connected...")
}
func (a *App) StartPGRead() {
	var err error
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	url := os.Getenv("POSTGRESQL_READ_URL")

	config, err := pgxpool.ParseConfig(url) 
	if err != nil {
		log.Fatal(ctx, "Failed to parse PostgreSQL config: ", err.Error())
	}

	config.MaxConns = 30                      
	config.MinConns = 5                       
	config.MaxConnLifetime = time.Hour        
	config.MaxConnIdleTime = 30 * time.Minute

	a.PGDBRead, err = pgxpool.NewWithConfig(ctx, config)

	if err != nil {
		log.Fatal(ctx, "Not connect DB Postgresql: ", err.Error())
	}

	if err := a.PGDBRead.Ping(ctx); err != nil {
		log.Fatal(ctx, "Failed to ping PostgreSQL Read: ", err.Error())
	}

	a.log.Info("DB Postgresql Readr was connected...")
}

func (a *App) stopPGWrite() {
	if a.PGDBWrite != nil {
		a.PGDBWrite.Close()
		a.log.Info("PG Write conexao finalizada ok")
	}
}

func (a *App) stopPGRead() {
	if a.PGDBRead != nil {
		a.PGDBRead.Close()
		a.log.Info("PG Read conexao finalizada ok")
	}
}

func (a *App) stopHttp() {
	a.httpServer.Close()
	a.log.Info("Http conexao finalizada ok")
}

func (a *App) Stop() {
	a.stopPGRead()
	a.stopPGWrite()
	// a.stopRedis()
	a.stopHttp()
	a.log.Info("Finalizado com sucesso")
}

