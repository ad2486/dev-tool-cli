# Dev CLI — Product Requirements Document

## Visão

Dev é uma ferramenta de linha de comando para configurar ambientes de desenvolvimento de maneira consistente e reproduzível.

O objetivo é que um desenvolvedor consiga preparar um computador novo executando poucos comandos, sem precisar pesquisar documentação de cada linguagem.

Exemplo:

```bash
dev install python
dev install rust
dev install node

dev init python
```

---

# Problema

Atualmente cada linguagem possui seu próprio processo de instalação.

Exemplos:

* Python + uv
* Rust + rustup
* Node + pnpm
* Go
* Java

Além disso, cada projeto exige configuração de formatter, linter, testes, gitignore, estrutura inicial etc.

Todo esse trabalho é repetitivo.

---

# Objetivos

* Unificar instalação de linguagens
* Padronizar criação de projetos
* Automatizar configuração inicial
* Possuir arquitetura extensível
* Ser multiplataforma
* Não depender de runtime externo

---

# Não objetivos (MVP)

* IDE própria
* Editor de código
* Gerenciador de versões (substituir asdf/mise)
* Hospedar templates online
* Cloud Sync

---

# Público

* Desenvolvedores iniciantes
* Desenvolvedores experientes
* Empresas
* Estudantes
* Pessoas configurando máquinas frequentemente

---

# Casos de uso

## Instalar linguagem

```bash
dev install python
```

Resultado:

* instala Python
* instala uv
* verifica PATH
* valida instalação

---

## Criar projeto

```bash
dev init python
```

Resultado:

* uv init
* git init
* README.md
* .gitignore
* estrutura padrão

---

## Diagnóstico

```bash
dev doctor
```

Verifica:

* Git
* Docker
* Linguagens
* PATH
* Ferramentas opcionais

---

## Atualização (pós-MVP)

```bash
dev update
```

Atualiza todas as ferramentas suportadas.

---

# Requisitos Funcionais

RF-001 Instalar linguagens.

RF-002 Inicializar projetos.

RF-003 Atualizar ferramentas (pós-MVP).

RF-004 Detectar sistema operacional.

RF-005 Detectar gerenciador de pacotes.

RF-006 Validar instalação.

RF-007 Possuir configurações do usuário.

RF-008 Suportar templates (MVP: templates embutidos usados pelo `init`; pós-MVP: comando `dev template`).

RF-009 Arquitetura baseada em providers.

---

# Requisitos Não Funcionais

* Inicialização rápida
* Baixo consumo de memória
* Binário único
* Código modular
* Fácil adição de linguagens
* Testável
* Multiplataforma

---

# Linguagens previstas

MVP:

* Python
* Rust

Pós-MVP:

* Node.js
* Go
* Java

---

# Estrutura de configuração

Cada linguagem possui um Provider.

Cada usuário possui um arquivo próprio de configuração.

Providers definem:

* instalação
* inicialização
* ferramentas padrão
* opções válidas

Usuário define apenas preferências (que sobrescrevem os defaults dos manifests — ver ARCHITECTURE.md).

---

# Comandos

MVP:

```bash
dev install <linguagem>
dev init <linguagem>
dev doctor
dev config get|set
```

Pós-MVP:

```bash
dev uninstall
dev update
dev search
dev template
```

## Flags globais (MVP)

* `--dry-run` — mostra os comandos que seriam executados, sem executá-los
* `--yes` / `-y` — não pede confirmação interativa (uso em scripts)
* `-v` / `-q` — aumenta / reduz a verbosidade do log
* `--config <arquivo>` — usa um arquivo de configuração alternativo

---

# Critérios de sucesso

* Instalar uma linguagem com um comando.
* Criar um projeto funcional em menos de 10 segundos.
* Adicionar uma nova linguagem sem alterar o núcleo da aplicação.
