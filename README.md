# AlGlobo: Trabajo Práctico Concurrentes

## AlGlobo

```sh
$ ./al-globo <configuracion.json>
```

### Ejemplo Archivo Configuración

```json
{
    "external_entities": [
        {
            "name": "AIRLINE",
            "ip": "127.0.0.1",
            "port": "3000"
        },
        {
            "name": "BANK",
            "ip": "127.0.0.1",
            "port": "3001"
        },
        {
            "name": "HOTEL",
            "ip": "127.0.0.1",
            "port": "3002"
        }
    ],
    "log_file": "payment-processor",
    "payment_transactions": "archivo.csv",
    "failed_transactions": "failed-transactions.csv"
}
```
