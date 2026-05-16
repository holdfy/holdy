package config

import (
	"log"
	"github.com/joho/godotenv"
)


func Init(){
	err := godotenv.Load()
	if err != nil {
		log.Fatalln("Erro lendo as variaveis de ambiente: ", err)
	}

}

