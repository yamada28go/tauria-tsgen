//- Generated from Append.rs

export interface Append {


    /**
* 
*/
    num: number;

    /**
* 
*/
    appendEx: AppendEx;


}


//- Generated from AppendEx.rs

export interface AppendEx {


    /**
* 
*/
    num: number;


}


//- Generated from Data.rs

export interface Data {


    /**
* 
*/
    msg: Message;

    /**
* 
*/
    append: Append;


}


//- Generated from Message.rs

export enum Message {


    /**
* Quit the application.
*/
    Quit = "Quit", // TODO: Handle different enum types (tuple, struct variants)

    /**
* Move to a new position.
*/
    Move = "Move", // TODO: Handle different enum types (tuple, struct variants)

    /**
* Write a message.
*/
    Write = "Write", // TODO: Handle different enum types (tuple, struct variants)

    /**
* Change the color.
*/
    ChangeColor = "ChangeColor", // TODO: Handle different enum types (tuple, struct variants)


}


