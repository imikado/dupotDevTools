# Base PostgreSQL de test (SQL Viewer)

```sh
docker build -t dupot-pg-example .
docker run --rm -d --name dupot-pg -p 5432:5432 dupot-pg-example
```

Paramètres de connexion dans le SQL Viewer :

| Champ    | Valeur    |
|----------|-----------|
| host     | localhost |
| port     | 5432      |
| database | devtools  |
| user     | devtools  |
| password | devtools  |

Tables : `author` (3 lignes) et `book` (6 lignes, `book.author_id` → `author.id`).

Requête de jointure :

```sql
SELECT a.name, b.title, b.published_on
FROM book b JOIN author a ON a.id = b.author_id
ORDER BY a.name, b.published_on;
```

Arrêt : `docker stop dupot-pg`
