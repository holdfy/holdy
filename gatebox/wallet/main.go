package main

import (
     "log"
     "palm-pay/config"
     "palm-pay/server"
     "os"

)

func main() {

	config.Init()

	app := server.New()
	app.Start()

	if err := app.Run(os.Getenv("PORT")); err != nil {
		log.Fatalf("%s", err.Error())
	}

defer app.Stop()
}

