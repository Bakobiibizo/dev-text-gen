#!/bin/bash

# Pull the Docker image
# docker pull ollama/ollama

# The index of the model to run, passed as the first argument to the script
index=$1

# Declare an associative array for model mapping
declare -A model_map=(
    [codellama]="codellama:70b-python"
    [llava]="llava:34b"
    [mixtral]="mixtral:instruct"
    [mistral]="mistral:instruct"
)

# List of models
model_list=(
    "codellama" 
    "llava" 
    "mixtral" 
    "mistral"
)

# Function to run Ollama container
run_ollama() {
    local idx=$1  # Use 'local' for function-scoped variables
    local model_name="${model_list[idx]}"  # Access array element
    local model="${model_map[$model_name]}"  # Access associative array value
    local container_name="ollama-ollama-${idx}"
    local port=$((11430 + idx)):11434  # Calculate port numbers
    local host="0.0.0.0"

    # Stop any existing container with the same name
    docker stop "$container_name" 2>/dev/null || true  # Ignore errors if container does not exist

    # Run the Docker container
    docker run --rm -d --name "$container_name" -p "$port" -v /root/.ollama:/root/.ollama "ollama/ollama" serve --host "$host" --model "$model"

    # Check the exit code of the last command
    local exit_code=$?
    return $exit_code
}

# Call the function with the provided index
run_ollama "$index"
exit_code=$?

# Exit the script with the exit code of the function
exit $exit_code
