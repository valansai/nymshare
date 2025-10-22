// MIT License
// Copyright (c) Valan Sai 2025
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use clap::Parser;


#[derive(Debug, Default)]
pub struct Config {
    pub serving_gateway: Option<String>,
    pub download_gateway: Option<String>,
}

#[derive(Parser, Debug)]
#[command(name = "NymShare")]
pub struct Args {
    #[arg(long, value_name = "GATEWAY_ADDRESS")]
    pub serving_gateway: Option<String>,

    #[arg(long, value_name = "DOWNLOAD_GATEWAY")]
    pub download_gateway: Option<String>,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Self {
            serving_gateway: args.serving_gateway,
            download_gateway: args.download_gateway,
        }
    }
}