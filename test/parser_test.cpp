#include <catch2/catch_test_macros.hpp>

#include "../src/lexer.h"
#include "../src/parser.h"

static std::vector<Token> tokens(std::string input) {
  Lexer l(input);
  return l.lex();
}

TEST_CASE("basic program can be parsed", "[parser]") {

  Parser p(tokens("return 123;"));

  ASTNodeProgram expected = (ASTNodeProgram){
      .body = {(ASTNodeStmt){
          .child = {(ASTNodeReturn){
              .expr =
                  (ASTNodeExpr){.child = {(ASTNodeTerm){
                                    .child = {(ASTNodeIntegerLiteral){
                                        .token = (Token){.type = TokenType::integer_literal,
                                                         .value = "123"},
                                    }}}}},
          }}}}};

  REQUIRE(p.parse() == expected);
}

TEST_CASE("can parse scopes", "[parser]") {
  Parser p(tokens("let x = 1; { let y = 2; }"));

  ASTNodeProgram expected = (ASTNodeProgram){
   .body = {
    (ASTNodeStmt){
     .child = {
      (ASTNodeLet){
       .identifier = (Token){.type = TokenType::identifier, .value = "x"},
       .expr =
        (ASTNodeExpr){
         .child = {
          (ASTNodeTerm){
           .child = {
            (ASTNodeIntegerLiteral){
             .token = (Token){.type = TokenType::integer_literal, .value = "1"},
            }
           }
          }
         }
        },
      }
     }
    },
    (ASTNodeStmt){
     .child = {
      (ASTNodeScope){
       .body = {
        (ASTNodeStmt){
         .child = {
          (ASTNodeLet){
           .identifier = (Token){.type = TokenType::identifier, .value = "y"},
           .expr =
            (ASTNodeExpr){
             .child = {
              (ASTNodeTerm){
               .child = {
                (ASTNodeIntegerLiteral){
                 .token = (Token){.type = TokenType::integer_literal, .value = "2"},
                }
               }
              }
             }
            },
          }
         }
        },
       }
      }
     }
    },
   }
  };

  REQUIRE(p.parse() == expected);
}
