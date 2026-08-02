_just:
    @just --list

# Full deploy: build images, load into cluster, apply manifests, restart pods
deploy:
    just ensure-cluster
    just build-and-add
    just apply-manifests
    just restart-deployments
    @echo "✅ Deploy complete — ft-lgtm is up and running"

# Create the k3d cluster if it doesn't already exist
ensure-cluster:
    #!/usr/bin/env bash
    if ! k3d cluster list | grep -q ft-lgtm; then
        k3d cluster create ft-lgtm -p "8080:80@loadbalancer"
    else
        echo "Cluster ft-lgtm already exists, skipping creation"
    fi

# Builds the required docker images and adds them to the ft-lgtm cluster
build-and-add:
    just build-images
    just add-images

# Build all Docker images
build-images:
    just build-frontend-image
    just build-backend-image

# Build frontend image
build-frontend-image:
    docker build -t ft-lgtm-frontend:local ./app/frontend

# Build backend image
build-backend-image:
    docker build -t ft-lgtm-backend:local ./app/backend

# Add docker images to the k8s cluster
add-images:
    just add-frontend-image
    just add-backend-image

# Add frontend image to cluster
add-frontend-image:
    k3d image import ft-lgtm-frontend:local -c ft-lgtm

# Add backend image to cluster
add-backend-image:
    k3d image import ft-lgtm-backend:local -c ft-lgtm

# Apply all k8s manifests (deployments, services, ingress)
apply-manifests:
    kubectl apply -f k8s/ -R

# Restart deployments so they pick up freshly imported images
restart-deployments:
    kubectl rollout restart deployment/ft-lgtm-frontend
    kubectl rollout restart deployment/ft-lgtm-backend
    kubectl rollout status deployment/ft-lgtm-frontend
    kubectl rollout status deployment/ft-lgtm-backend
