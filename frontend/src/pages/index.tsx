// import { error } from "console";
import React, { ChangeEvent, useState} from "react";


export default function Home(){ 

  const [inputEmail, setEmail] = useState("");
  const [inputPassword, setPassword] = useState("");
  const [wrongEmail, setEmailAlert] = useState(false);
  const [wrongLogin, setLoginAlert] = useState(false);
  // const [errorMessage, setError] = useState("");
  const serverURL = "7878-peytonjohns-realtimecha-eashxnzgboj.ws-us116.gitpod.io";

  return (
    <div className="flex bg-slate-500 h-screen w-screen justify-center">
      <div className="block space-y-auto">
        <h1 className="text-center mt-[5%] text-2xl font-bold">Log in</h1>
        <div className="mb-[5%]">
          <h2 className="text-xl">Email:</h2>
          <input value={inputEmail} onChange={onEmailChange} className="bg-slate-700 w-full" type="text" placeholder="john.doe@company.com" required></input>
          {wrongEmail ? <ErrorMessage text={"Invalid Email"}/> : null}
        </div>
        <div className="mb-[5%]">
          <h2 className="text-xl">Password:</h2>
          <input value={inputPassword} onChange={onPasswordChange} className="bg-slate-700 w-full" type="password" placeholder="•••••••••" required ></input>
        </div>
        <div className="align-center">
          <button onClick={logIn} className="hover:bg-slate-700 bg-slate-800 rounded w-full max-h-16 min-h-8 max-w-24">Log in</button>
          {wrongLogin ? <ErrorMessage text={"Invalid Login"}/> : null}
          <p className="text-center">If you dont have an account, click <a href="sign-up" className="text-sky-400">here</a> to sign up</p>
        </div>
        <div>User password: {inputPassword}</div>
        {/* <div><p>Error: {errorMessage}</p></div> */}
      </div>
    </div>
  );
  
  function onEmailChange(e:  ChangeEvent<HTMLInputElement>){
    setEmail(e.target.value);
  }

  function onPasswordChange(e: ChangeEvent<HTMLInputElement>){
    setPassword(e.target.value);
  }

  async function logIn(){
    // const emailRegex = /^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$/g;
    if (inputEmail == "" || inputPassword == ""){
      setLoginAlert(true);

    // }else if (emailRegex.test(inputEmail.trim())){
    //   setEmailAlert(true);
    }else{
      setLoginAlert(false);
      setEmailAlert(false);
      await fetch(serverURL + "/users", {
        // mode: "no-cors",
        method: "POST",
        headers: new Headers(
          {
            // 'content-type': 'application/json',
            'Authorization' : "Basic " + JSON.stringify({
              email: inputEmail,
              password: inputPassword,
            }),
          }
        ),
      }).then((response) => {
        if (response.status == 401){
          setLoginAlert(true)
        }else if (response.status == 200){
          window.location.href = window.location.href + "/chat"
        }
      })

      
      
    }
  }
}

function ErrorMessage(text: {text: string}){
  return (
    <p className="text-red-600 bg-red-400 border-2 border-red-600 mt-[2%] "> Error: {text.text} </p>
  )
}


