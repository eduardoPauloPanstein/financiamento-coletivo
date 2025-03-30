use crowdfunding::crowdfunding_proxy; // Proxy para interagir com o contrato inteligente.
use multiversx_sc_scenario::imports::*; // Ferramentas para simulação de blockchain.

const CODE_PATH: MxscPath = MxscPath::new("output/crowdfunding.mxsc.json");

// Função para configurar o ambiente de simulação do blockchain
fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    // Define o diretório atual como o workspace que contém o contrato
    blockchain.set_current_dir_from_workspace("crowdfunding");

    // Registra o contrato Crowdfunding no ambiente de simulação
    blockchain.register_contract(CODE_PATH, crowdfunding::ContractBuilder);

    blockchain
}

const OWNER: TestAddress = TestAddress::new("owner");
const CROWDFUNDING_ADDRESS: TestSCAddress = TestSCAddress::new("crowdfunding");

#[test]
fn crowdfunding_deploy_test() {
    // Definindo o ambiente de simulação, struct para executar cenarios de contatos
    let mut world = world();

    // Inicializa a conta do proprietário com um nonce e um saldo inicial
    world.account(OWNER).nonce(0).balance(1_000_000);

    // Realiza o deploy do contrato Crowdfunding
    let crowdfunding_address = world
        .tx() // Inicia uma nova transação
        .from(OWNER) // Especifica o remetente da transação
        .typed(crowdfunding_proxy::CrowdfundingProxy) // Especifica o tipo do contrato
        .init(500_000_000_000u64) // Inicializa o contrato com a meta de arrecadação
        .code(CODE_PATH) // Fornece o código compilado do contrato
        .new_address(CROWDFUNDING_ADDRESS) // Especifica o novo endereço do contrato
        .returns(ReturnsNewAddress) // Espera que a transação retorne o novo endereço
        .run(); // Executa a transação

    // Verifica se o contrato foi implantado no endereço esperado
    assert_eq!(crowdfunding_address, CROWDFUNDING_ADDRESS.to_address());

    // Verifica o saldo da conta do proprietário após o deploy
    world.check_account(OWNER).balance(1_000_000);

    // Faz uma consulta ao contrato para verificar seu estado inicial
    world
        .query() // Inicia uma nova consulta
        .to(CROWDFUNDING_ADDRESS) // Especifica o endereço do contrato a ser consultado
        .typed(crowdfunding_proxy::CrowdfundingProxy) // Especifica o tipo do contrato
        .target() // Define o alvo da consulta como o contrato
        .returns(ExpectValue(500_000_000_000u64)) // Espera que a meta de arrecadação corresponda ao valor inicializado
        .run(); // Executa a consulta
}

// Comandos terminal ~/crowdfunding:
// sc-meta all build
// cargo test


/*
# Ajuda para entender este fonte, código de teste do contrato Crowdfunding

## Estrutura do Código
1. **Importações**:
   - `crowdfunding::crowdfunding_proxy`: Este módulo fornece uma interface (proxy) para interagir com o contrato inteligente. Ele simplifica chamadas e consultas ao contrato.
   - `multiversx_sc_scenario::imports::*`: Ferramentas para criar um ambiente de simulação de blockchain, permitindo testar contratos inteligentes localmente.

2. **Constantes**:
   - `CODE_PATH`: Caminho para o arquivo JSON que contém o código compilado do contrato inteligente. Este arquivo é gerado após a compilação do contrato.
   - `OWNER`: Representa o endereço do proprietário que realizará o deploy do contrato. É como uma "carteira" no blockchain.
   - `CROWDFUNDING_ADDRESS`: Endereço onde o contrato será implantado. Este é o identificador único do contrato no blockchain.

3. **Função `world`**:
   - Configura o ambiente de simulação do blockchain.
   - Define o diretório de trabalho onde o contrato está localizado.
   - Registra o contrato no ambiente de simulação para que ele possa ser usado nos testes.

4. **Teste `crowdfunding_deploy_test`**:
   - Este é o teste principal que verifica se o contrato Crowdfunding pode ser implantado corretamente.
   - Ele também verifica se o estado inicial do contrato (como a meta de arrecadação) está configurado corretamente.
   - Aqui não tem caller, pagamento, preço do gás ou limite de gás. No blockchain real, uma consulta a um contrato inteligente não cria uma transação no blockchain, portanto, nenhuma conta é necessária.

## Conceitos Importantes
- **Deploy do Contrato**:
  - O deploy é o processo de "instalar" o contrato no blockchain. Isso é feito através de uma transação (`tx`) que carrega o código do contrato e o inicializa com os parâmetros necessários (como a meta de arrecadação).
  - Após o deploy, o contrato recebe um endereço único no blockchain, que pode ser usado para interagir com ele.

- **Consultas ao Contrato**:
  - As consultas (`query`) permitem verificar informações no contrato sem alterar seu estado. Por exemplo, você pode consultar a meta de arrecadação ou o saldo atual sem criar uma transação.
  - No exemplo, a consulta verifica se a meta de arrecadação foi configurada corretamente após o deploy.

- **Testes**:
  - Testes são uma forma segura de verificar se o contrato funciona como esperado antes de implantá-lo em um blockchain real.
  - Sempre verifique o estado inicial e final após operações importantes, como deploys ou transações.

## Exemplos Práticos
  - Mude o valor `500_000_000_000u64` na inicializacao ou na consulta e veja o teste falhar.

  - Crie testes para verificar outras funcionalidades do contrato, como contribuições de usuários ou encerramento da campanha.

*/