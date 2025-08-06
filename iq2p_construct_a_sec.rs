#[derive(Debug, Deserialize, Serialize)]
struct PipelineConfig {
    repo_url: String,
    branch: String,
    environment: Environment,
    steps: Vec<Step>,
}

#[derive(Debug, Deserialize, Serialize)]
enum Environment {
    Dev,
    Stg,
    Prod,
}

#[derive(Debug, Deserialize, Serialize)]
enum Step {
    Build {
        builder: Builder,
        args: Vec<String>,
    },
    Deploy {
        target: DeployTarget,
        config: DeployConfig,
    },
}

#[derive(Debug, Deserialize, Serialize)]
enum Builder {
    Docker,
    Maven,
    Gradle,
}

#[derive(Debug, Deserialize, Serialize)]
enum DeployTarget {
    Kubernetes,
    CloudFoundry,
    AWS,
}

#[derive(Debug, Deserialize, Serialize)]
struct DeployConfig {
    artifact: String,
    namespace: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: PipelineConfig = serde_yaml::from_str(include_str!("config.yaml"))?;

    let mut pipeline = String::new();

    pipeline.push_str("pipeline {\n");
    pipeline.push_str("    agent {\n");
    pipeline.push_str("        docker { image 'maven:3.6.0'\n");
    pipeline.push_str("    }\n");
    pipeline.push_str("}\n");

    for step in config.steps {
        match step {
            Step::Build { builder, args } => {
                pipeline.push_str("    stage('Build') {\n");
                match builder {
                    Builder::Docker => pipeline.push_str("        sh 'docker build .`\n"),
                    Builder::Maven => pipeline.push_str("        sh 'mvn clean package'\n"),
                    Builder::Gradle => pipeline.push_str("        sh 'gradle build'\n"),
                }
                for arg in args {
                    pipeline.push_str(&format!("        sh '{}'\n", arg));
                }
                pipeline.push_str("    }\n");
            }
            Step::Deploy { target, config } => {
                pipeline.push_str("    stage('Deploy') {\n");
                match target {
                    DeployTarget::Kubernetes => pipeline.push_str("        kubernetesDeploy {\n"),
                    DeployTarget::CloudFoundry => pipeline.push_str("        cloudFoundryDeploy {\n"),
                    DeployTarget::AWS => pipeline.push_str("        awsDeploy {\n"),
                }
                pipeline.push_str(&format!("            artifact '{}'\n", config.artifact));
                pipeline.push_str(&format!("            namespace '{}'\n", config.namespace));
                pipeline.push_str("        }\n");
                pipeline.push_str("    }\n");
            }
        }
    }

    pipeline.push_str("}\n");

    Ok(std::fs::write("Jenkinsfile", pipeline)?)?;
}