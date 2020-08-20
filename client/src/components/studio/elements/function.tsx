import React from "react"
import { Card, Heading, Button, Content, Columns } from "react-bulma-components"

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faEquals } from "@fortawesome/free-solid-svg-icons"

import Argument from "./argument"
import Variable from "./variables/variable"

const Function = (props: {
  id: string
  value: any
  handleElement: Function
  getElement: Function
  context: { parent: string; current: string }
}) => {
  return (
    <Card>
      <Card.Header>
        <Card.Header.Icon className="has-text-primary">
          <FontAwesomeIcon icon={faEquals} size="2x" />
        </Card.Header.Icon>
        <Card.Header.Title>{props?.value?.name}</Card.Header.Title>
      </Card.Header>
      <Card.Content>
        <Columns centered={true}>
          <Columns.Column size="half">
            <Content className="has-text-centered">
              <Heading size={4}>Input</Heading>
            </Content>
            {props?.value?.inputs?.map((elementId: string, index: number) => {
              return (
                <Argument
                  key={index}
                  element={props?.getElement(elementId)}
                  disabled={{ settings: false, edit: true }}
                  handleElement={props.handleElement}
                  getElement={props.getElement}
                />
              )
            })}
          </Columns.Column>
          <Columns.Column size="half">
            <Content className="has-text-centered">
              <Heading size={4}>Output</Heading>
            </Content>
            {props?.value?.outputs?.map((elementId: string, index: number) => {
              return (
                <Variable
                  key={index}
                  element={props?.getElement(elementId)}
                  disabled={{ settings: false, edit: true }}
                  handleElement={props.handleElement}
                  getElement={props.getElement}
                />
              )
            })}
          </Columns.Column>
        </Columns>
      </Card.Content>
    </Card>
  )
}

export default Function
