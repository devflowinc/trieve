# Makefile

default:
	@if [ ! -f .env ]; then \
		cp .env.example .env; \
		echo "File .env created based on .env.example"; \
		docker compose up -d; \
	else \
		docker compose up -d; \
	fi

down:
	docker compose down

# Remove volumes
clean: down
	@docker volume ls | grep trieve | awk '{print $$2}' | while read volume; do docker volume rm $$volume; done

# Show logs
logs:
	docker compose logs -f --tail=100

