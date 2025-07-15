md
# Lotto Analysis Thai

This project aims to provide tools and analysis for the Thai lottery.

## Key Features & Benefits

*   **Web Interface:** User-friendly web interface built with HTML, CSS, and JavaScript.
*   **Data Scraping:** Functionality to scrape lottery results data (implementation details in `static/app.js`).
*   **Backend Logic:** Utilizes Rust for backend processing (implementation details in `src/main.rs`).
*   **Containerization:** Packaged as a Docker container for easy deployment.

## Prerequisites & Dependencies

Before you begin, ensure you have the following installed:

*   **Rust:** Rust toolchain for building the backend. Get it from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).
*   **Docker:**  For containerizing and deploying the application.  Download from [https://www.docker.com/get-started/](https://www.docker.com/get-started/).
*   **Cargo:**  Rust's package manager (installed with Rust).

## Installation & Setup Instructions

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/Jittakorn-S/LottoAnalysisThai.git
    cd LottoAnalysisThai
    ```

2.  **Build the Docker image:**
    ```bash
    docker build -t lotto-analysis .
    ```

3.  **Run the Docker container:**
    ```bash
    docker run -p 8080:8080 lotto-analysis
    ```
    *(This assumes the application is exposed on port 8080. Adjust if necessary.)*

4.  **Access the application:**
    Open your web browser and navigate to `http://localhost:8080`.

## Usage Examples & API Documentation

The `static/app.js` file handles the front-end interactions.  Key functionalities include:

*   **Scrape Button:** Triggers the data scraping process (implementation depends on the Rust backend).
*   **Progress Display:** Updates the UI with the scraping progress.
*   **Table Display:** Presents the scraped data in a tabular format.

Details on the API endpoints and data formats would be provided here if the backend exposed an API. Since the provided files focus on the frontend and dockerization, API details are not available at this time.

## Configuration Options

The application can be configured through the following options:

*   **Port Mapping (Docker):**  The `-p` flag in the `docker run` command allows you to map the container's port to a different port on your host machine. For example, `-p 80:8080` maps the container's port 8080 to your host's port 80.

*   **Environment Variables:** If the Rust backend uses environment variables for configuration (e.g., API keys, database connection strings), you can pass them to the Docker container using the `-e` flag:
    ```bash
    docker run -e API_KEY=your_api_key -p 8080:8080 lotto-analysis
    ```

*   **`render.yaml`:** The presence of this file suggests potential deployment on Render.com. Configuration of the application through render.com is done via this file.

## Contributing Guidelines

We welcome contributions to the Lotto Analysis Thai project!

1.  Fork the repository.
2.  Create a new branch for your feature or bug fix.
3.  Make your changes and commit them with clear and descriptive commit messages.
4.  Submit a pull request to the main branch.

## License Information

License information is not specified in the provided data.

## Acknowledgments

This project utilizes the following technologies:

*   **JavaScript**
*   **Rust**
*   **Docker**
