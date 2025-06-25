import * as cdk from 'aws-cdk-lib'
import {Construct} from 'constructs'
import {getExtensionLayerArn} from './get-extension-layer-arn'
import {DecisionAlgorithmType, OptimeistProps} from './types'

export class NodejsFunction extends cdk.aws_lambda_nodejs.NodejsFunction {
  constructor(
    scope: Construct,
    id: string,
    props: cdk.aws_lambda_nodejs.NodejsFunctionProps & {
      optimeistProps: OptimeistProps
    },
  ) {
    const memory = new cdk.aws_ssm.StringParameter(scope, id + 'Memory', {
      description: 'Memory size for the Lambda function',
      stringValue: (props.memorySize || 128).toString(),
    })

    super(scope, id, {
      ...props,
      memorySize: cdk.Token.asNumber(memory.stringValue),
      environment: {
        ...props.environment,
        OPTIMEIST_MEMORY_PARAMETER_NAME: memory.parameterName,
        OPTIMEIST_DECISION_ALGORITHM_TYPE: props.optimeistProps.decisionAlgorithmType || DecisionAlgorithmType.BALANCED,
      },
    })

    const accessTokenSecret = cdk.aws_secretsmanager.Secret.fromSecretNameV2(
      this,
      'OptimeistAccessTokenSecret',
      props.optimeistProps.accessTokenSecretName,
    )

    this.addEnvironment('OPTIMEIST_ACCESS_TOKEN_SECRET_ARN', accessTokenSecret.secretArn)

    /**
     * Grant the Lambda function write access to the parameter for autoupdates from the lambda layer
     */
    memory.grantWrite(this)

    /**
     * Grant the Lambda function read access to the secret for the access token
     */
    accessTokenSecret.grantRead(this)

    /**
     * Grant the Lambda function permission to update its own configuration.
     * Required for update from the lambda layer.
     */
    new cdk.aws_iam.Policy(this, 'SelfUpdatePolicy', {
      roles: [this.role!],
      statements: [
        new cdk.aws_iam.PolicyStatement({
          actions: ['lambda:UpdateFunctionConfiguration', 'lambda:GetFunction'],
          resources: [this.functionArn],
        }),
      ],
    })

    this.addLayers(
      cdk.aws_lambda.LayerVersion.fromLayerVersionArn(
        this,
        'OptimeistLayer',
        getExtensionLayerArn({
          architecture: this.architecture,
        }),
      ),
    )
  }
}
