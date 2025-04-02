use crowdfunding::crowdfunding_proxy::{self, Status}; // Proxy para interagir com o contrato inteligente.
use multiversx_sc_scenario::imports::*; // Ferramentas para simulação de blockchain.

// Caminho para o código compilado do contrato inteligente
const CODE_PATH: MxscPath = MxscPath::new("output/crowdfunding.mxsc.json");

// Endereço do proprietário do contrato (quem realiza o deploy)
const OWNER: TestAddress = TestAddress::new("owner");

// Endereço do contrato inteligente no ambiente de simulação
const CROWDFUNDING_ADDRESS: TestSCAddress = TestSCAddress::new("crowdfunding");

// Endereço de um doador que participará dos testes
const DONOR: TestAddress = TestAddress::new("donor");

/// Configura o ambiente de simulação do blockchain.
///
/// # Detalhes
/// - Cria um ambiente de simulação para testes.
/// - Define o diretório de trabalho como o local onde o contrato está.
/// - Registra o contrato Crowdfunding no ambiente de simulação.
///
/// # Retorno
/// - Um objeto `ScenarioWorld` que representa o ambiente de simulação.
fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    // Define o diretório atual como o workspace que contém o contrato
    blockchain.set_current_dir_from_workspace("crowdfunding");

    // Registra o contrato Crowdfunding no ambiente de simulação
    blockchain.register_contract(CODE_PATH, crowdfunding::ContractBuilder);

    blockchain
}

/// Realiza o deploy do contrato Crowdfunding no ambiente de simulação.
///
/// # Detalhes
/// - Inicializa o contrato com uma meta de arrecadação e um prazo.
/// - Define o saldo inicial do proprietário.
/// - Verifica se o contrato foi implantado no endereço esperado.
///
/// # Retorno
/// - Um objeto `ScenarioWorld` com o contrato implantado.
fn crowdfunding_deploy() -> ScenarioWorld {
    let mut world = world();

    // Define o saldo inicial do proprietário
    world.account(OWNER).nonce(0).balance(1_000_000);

    // Realiza o deploy do contrato com os parâmetros iniciais
    let crowdfunding_address = world
        .tx()
        .from(OWNER)
        .typed(crowdfunding_proxy::CrowdfundingProxy)
        .init(500_000_000_000u64, 123_000u64) // Meta: 500 EGLD, Prazo: 123_000
        .code(CODE_PATH)
        .new_address(CROWDFUNDING_ADDRESS)
        .returns(ReturnsNewAddress)
        .run();

    // Verifica se o contrato foi implantado no endereço esperado
    assert_eq!(crowdfunding_address, CROWDFUNDING_ADDRESS.to_address());

    world
}

/// Testa o deploy do contrato Crowdfunding.
///
/// # Detalhes
/// - Verifica se o contrato foi implantado corretamente.
/// - Confirma se os valores iniciais (meta e prazo) estão configurados corretamente.
#[test]
fn crowdfunding_deploy_test() {
    let mut world = crowdfunding_deploy();

    // Verifica o saldo inicial do proprietário
    world.check_account(OWNER).balance(1_000_000);

    // Verifica se a meta de arrecadação foi configurada corretamente
    world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_proxy::CrowdfundingProxy)
        .target()
        .returns(ExpectValue(500_000_000_000u64))
        .run();

    // Verifica se o prazo foi configurado corretamente
    world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_proxy::CrowdfundingProxy)
        .deadline()
        .returns(ExpectValue(123_000u64))
        .run();
}

/// Realiza uma doação ao contrato Crowdfunding.
///
/// # Detalhes
/// - Define o saldo inicial do doador.
/// - Realiza uma transação de doação para o contrato.
///
/// # Retorno
/// - Um objeto `ScenarioWorld` com a doação registrada.
fn crowdfunding_fund() -> ScenarioWorld {
    let mut world = crowdfunding_deploy();

    // Define o saldo inicial do doador
    world.account(DONOR).nonce(0).balance(400_000_000_000u64);

    // Realiza uma doação de 250 EGLD
    world
        .tx()
        .from(DONOR)
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_proxy::CrowdfundingProxy)
        .fund()
        .egld(250_000_000_000u64)
        .run();

    world
}

/// Testa a funcionalidade de doação ao contrato.
///
/// # Detalhes
/// - Verifica se o saldo do doador e do contrato foram atualizados corretamente.
/// - Confirma se o depósito do doador foi registrado no contrato.
#[test]
fn crowdfunding_fund_test() {
    let mut world = crowdfunding_fund();

    // Verifica o saldo do proprietário (não deve mudar)
    world.check_account(OWNER).nonce(1).balance(1_000_000u64);

    // Verifica o saldo do doador após a doação
    world
        .check_account(DONOR)
        .nonce(1)
        .balance(150_000_000_000u64);

    // Verifica o saldo do contrato após a doação
    world
        .check_account(CROWDFUNDING_ADDRESS)
        .nonce(0)
        .balance(250_000_000_000u64);

    // Verifica se a meta de arrecadação permanece inalterada
    world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_proxy::CrowdfundingProxy)
        .target()
        .returns(ExpectValue(500_000_000_000u64))
        .run();

    // Verifica se o prazo permanece inalterado
    world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_proxy::CrowdfundingProxy)
        .deadline()
        .returns(ExpectValue(123_000u64))
        .run();

    // Verifica se o depósito do doador foi registrado corretamente
    world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_proxy::CrowdfundingProxy)
        .deposit(DONOR)
        .returns(ExpectValue(250_000_000_000u64))
        .run();
}

/// Testa a tentativa de doação após o prazo.
///
/// # Detalhes
/// - Avança o tempo para além do prazo.
/// - Verifica se a doação é rejeitada.
/// - Confirma que o status do contrato é `Failed`.
#[test]
fn crowdfunding_fund_too_late_test() {
    let mut world = crowdfunding_fund();

    // Avança o tempo para além do prazo
    world.current_block().block_timestamp(123_001u64);

    // Tenta realizar uma doação após o prazo
    world
        .tx()
        .from(DONOR)
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_proxy::CrowdfundingProxy)
        .fund()
        .egld(10_000_000_000u64)
        .with_result(ExpectError(4, "cannot fund after deadline"))
        .run();

    // Verifica se o status do contrato é `Failed`
    world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_proxy::CrowdfundingProxy)
        .status()
        .returns(ExpectValue(Status::Failed))
        .run();
}



/*
## Conceitos Importantes

### Deploy do Contrato
- O deploy é o processo de "instalar" o contrato no blockchain. Isso é feito através de uma transação (`tx`) que carrega o código do contrato e o inicializa com os parâmetros necessários (como a meta de arrecadação e o prazo).
- Após o deploy, o contrato recebe um endereço único no blockchain, que pode ser usado para interagir com ele.
- No ambiente de simulação, o deploy é realizado com a função `crowdfunding_deploy`, que também verifica se o contrato foi implantado corretamente.

### Consultas ao Contrato
- As consultas (`query`) permitem verificar informações no contrato sem alterar seu estado. Por exemplo:
  - Consultar a meta de arrecadação (`target`).
  - Consultar o prazo final (`deadline`).
  - Consultar o depósito de um doador específico (`deposit`).
- No blockchain real, consultas não criam transações e não consomem gás, mas no ambiente de simulação, elas são usadas para validar o estado do contrato.

### Testes
- Testes são fundamentais para garantir que o contrato funciona como esperado antes de ser implantado em um blockchain real.
- Cada teste deve verificar o estado inicial e final após operações importantes, como deploys, doações ou saques.
- Exemplos de testes implementados:
  - `crowdfunding_deploy_test`: Verifica se o contrato foi implantado corretamente.
  - `crowdfunding_fund_test`: Verifica se as doações são registradas corretamente.
  - `crowdfunding_fund_too_late_test`: Verifica se doações após o prazo são rejeitadas.

## Exemplos Práticos

### Alterar a Meta de Arrecadação
- Modifique o valor `500_000_000_000u64` na inicialização do contrato (função `init`) para outro valor, como `1_000_000_000_000u64`.
- Execute os testes novamente. O teste `crowdfunding_deploy_test` deve falhar, pois espera que a meta seja `500_000_000_000u64`.

### Adicionar Novos Testes
- Crie um teste para verificar o comportamento do contrato quando o status é `Successful`. Por exemplo:
  - Simule várias doações que atinjam ou excedam a meta.
  - Verifique se o proprietário pode sacar os fundos.
  - Verifique se o status do contrato é alterado para `Successful`.

### Comandos terminal ~/crowdfunding:
1. Compile o contrato: `sc-meta all build`
2. Gere o proxy: `sc-meta all proxy`
3. Execute os testes: `sc-meta test`
*/