import React, {ChangeEvent, useState}  from "react";


export default function Home() {
  const [inputMessage, setMessage] = useState("");
  const serverURL = "7878-peytonjohns-realtimecha-eashxnzgboj.ws-us116.gitpod.io";

  return (
    <div className="block size-full">
      <h1 className="text-center text-4xl">My Chat App</h1>
      <div className = "bg-slate-900  w-1/2 h-full m-auto mt-8 flex p-3 rounded">
        <div className="size-1/4 max-h-96 overflow-y-auto">
        <h1>Chat Rooms</h1>
        <div className="rounded bg-slate-950">
          <ul className="">
            <li><ChatRoomButton /></li>
            <li><ChatRoomButton /></li>
            <li><ChatRoomButton /></li>
            <li><ChatRoomButton /></li>
            <li><ChatRoomButton /></li>
            <li><ChatRoomButton /></li>
          </ul>
        </div>
        </div>
        <div>
          <div className="ml-2 rounded bg-slate-950 size-3/4 h-full p-2 block space-y-2 flex-col max-h-96 overflow-y-auto ">
          <MessageBox />
          <LongerMessageBox />
          <LongerMessageBox />
          <MessageBox />
          <LongerMessageBox />
          <MessageBox />
          <LongerMessageBox />
          <MessageBox />
          <LongerMessageBox />
          <MessageBox />
          <LongerMessageBox />
          <MessageBox />
          <LongerMessageBox />
          </div>
          <div className="h-38">
            <input type="text" onChange={OnMessageBoxChange} className="text-black size-auto h-max rounded bg-slate-600 mt-5 ml-2 col-span-2"></input>
            <button onClick={sendMessage} className="text-white rounded bg-slate-950 p-2 w-[10%] col-span-1"> Send </button>
          </div>
          {/* Work on CSS later */}
        </div>
      </div>
    </div>
  );

  function OnMessageBoxChange(e: ChangeEvent<HTMLInputElement>){
    setMessage(e.target.value)
  }

  async function sendMessage(){
    if (inputMessage == ""){
      
    }else {
      await fetch(serverURL + "/chats", {
        // mode: "no-cors",
        method: "PUT",
        // headers: new Headers(
        //   {
            
        //   }
        // ),
        body: inputMessage,
      })
    }
  }
}

function MessageBox(){
  return (
    <div className="bg-slate-400 rounded-lg p-1 size-fit">
      <h1 className="text-black">Message Here</h1>
    </div>
  );
}

function ChatRoomButton(){
  return (
    <button className="hover:bg-slate-700 rounded w-full max-h-16 min-h-8"> Chat Room </button>
  );
}

function LongerMessageBox(){
  return (
    <div className="bg-slate-400 rounded-lg p-1 size-fit">
      <h1 className="text-black"> Super Super Super Super Super Super Super Super Super long message</h1>
    </div>
  );
}




