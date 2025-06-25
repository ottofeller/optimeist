export enum DecisionAlgorithmType {
  COST = 'COST',
  SPEED = 'SPEED',
  BALANCED = 'BALANCED',
}

export type OptimeistProps = {
  /**
   * The name of the secret containing the access token
   * The secret must be a plaintext secret
   */
  accessTokenSecretName: string

  /**
   * The decision algorithm to use
   *
   * @default DecisionAlgorithmType.BALANCED
   */
  decisionAlgorithmType?: DecisionAlgorithmType
}
