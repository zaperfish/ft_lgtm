.PHONY: install-deps install-k3d install-kubectl install-just

install-deps: install-k3d install-kubectl install-just
	@echo "All dependencies are ready."

install-k3d:
	@if command -v k3d >/dev/null 2>&1; then \
		echo "k3d is already installed: $$(k3d version | head -n 1)"; \
	else \
		echo "Installing k3d..."; \
		curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash; \
	fi

install-kubectl:
	@if command -v kubectl >/dev/null 2>&1; then \
		echo "kubectl is already installed:"; \
		echo "$$(kubectl version 2>/dev/null)"; \
	else \
		echo "Installing kubectl..."; \
		curl -LO "https://dl.k8s.io/release/$$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/$$(uname | tr '[:upper:]' '[:lower:]')/amd64/kubectl"; \
		chmod +x kubectl; \
		sudo mv kubectl /usr/local/bin/kubectl; \
	fi

install-just:
	@if command -v just >/dev/null 2>&1; then \
		echo "just is already installed: $$(just --version)"; \
	else \
		echo "Installing just..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin; \
	fi
