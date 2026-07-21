# Ecossistema USBRAIN

## Visao rapida

O USBRAIN e um ecossistema de componentes interoperaveis, nao um unico executavel isolado.

## Builder

Aplicacao desktop em `Tauri + Rust` responsavel por:

- detectar destinos;
- gerar build plans e manifestos;
- montar o USBRAIN no destino;
- validar estrutura e runtime;
- executar probes e gates;
- preparar pacote de distribuicao e release.

## Runtime

Camada executada a partir do USB, responsavel por:

- detectar hardware hospedeiro;
- selecionar o melhor modelo local;
- iniciar interface e servicos;
- respeitar politicas offline, locais e hibridas;
- consumir configuracoes geradas pelo Builder.

## PACKs

PACKs sao modulos especialistas que expandem o valor do USBRAIN com:

- conhecimento curado;
- comportamento;
- ferramentas;
- combinacoes hibridas dos itens anteriores.

## Hub

O Hub e a futura camada comunitaria para:

- publicacao de PACKs;
- descoberta por categoria, idioma e compatibilidade;
- distribuicao de packs oficiais e comunitarios;
- curadoria inicial do ecossistema.

## Catalogo de Modelos

Camada viva para descoberta e recomendacao de modelos open source, incluindo:

- metadados de idioma;
- tamanho em disco;
- RAM estimada;
- afinidade com uso offline;
- adequacao a CPU e GPU;
- recomendacao contextual por alvo.

## Como tudo se conecta

1. O Builder analisa o destino e monta o USBRAIN.
2. O Runtime executa esse USBRAIN em hardware variavel.
3. PACKs especializam a experiencia.
4. O Hub distribui e organiza esses PACKs.
5. O Catalogo de Modelos impede que o Builder envelheca com lista fixa.

## Estado atual

Hoje, o foco principal do projeto esta em:

- Builder funcional;
- Runtime validado;
- contratos claros;
- fluxo de distribuicao e release publicavel;
- base pronta para abertura publica no GitHub.
