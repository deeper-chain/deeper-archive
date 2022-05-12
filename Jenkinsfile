#!groovy
def slackChannel = '#devops-test'
def execNode = 'common-runner'
def ansiblePath = '/home/ubuntu'
def upstreamProjects = ''

if (env.BRANCH_NAME == "master") {
    deployCmd = ""
}


pipeline {
    agent {
        node { label execNode }
    }

    options {
        disableConcurrentBuilds()
        buildDiscarder(logRotator(numToKeepStr: '5', artifactNumToKeepStr: '3'))
    }

    triggers {
        upstream(
            upstreamProjects: upstreamProjects,
            threshold: hudson.model.Result.SUCCESS
        )
    }
    environment {
        webhook_key = credentials('webhook_key')
        ANSIBLE_PATH = "${ansiblePath}"
    }

    stages {
        stage('test') {
            when {
                not {
                    anyOf {
                        branch 'master'
                    }
                }
            }
            stages {
                stage('Unit Test') {
                    steps {
                        echo 'prepare to code test'
                    //TODO
                    }
                }
                stage('report') {
                    when {
                        not {
                            branch 'PR-*'
                        }
                    }
                    steps {
                        echo 'generate code report'
                    //TODO
                    }
                }
            }
        }

        stage('Build') {
            when {
                anyOf {
                   branch "master"
                   branch "dev"
                }
            }
            steps {
                sh '''
                    sudo cargo build
                    '''
            }
        }

        stage('Deploy code') {
            when {
                anyOf {
                   branch "master"
                   branch "dev"
                }
            }
            steps {
                sh 'sudo cp ./target/debug/deeper-archive $ANSIBLE_PATH/'
                sh 'ANSIBLE_HOST_KEY_CHECKING=False ansible-playbook -i $ANSIBLE_PATH/hosts $ANSIBLE_PATH/playbooks/deploy-dev.yml'
            }
        }
    }
    post {
        always {
           cleanWs()
        }
        success {
            slackSend channel: slackChannel, color: 'good',
                message: "${env.JOB_NAME} CICD SUCCESS,<${env.BUILD_URL}console|cliek me get details>"
        }
        failure {
            slackSend channel: slackChannel, color: 'danger',
                message: "${env.JOB_NAME} CICD FAILED!!! <${env.BUILD_URL}console|cliek me check log>"
        }
    }

}